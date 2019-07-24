use crate::data::CUM;
use itertools_num::linspace;
use lazy_static::*;
use plotters::prelude::*;
use rand::prelude::*;
use rand_distr::{Beta, Exp, Uniform};
use rand_xorshift::XorShiftRng;

/// Bounds points on distribution plot.
fn bound(x: f32) -> f32 {
    if x < 0. {
        0.
    } else if x > 1. {
        1.
    } else {
        x
    }
}

/// Different types of provided distributions
pub enum DistMode {
    /// Mixture of Paretos where the `q` parameter is sampled uniformly from [0.001, 1)
    Uniform,
    /// A single Pareto where the `q` parameter is provided by the user.
    Q(f32),
    /// Mixture of Paretos where the `q` parameter is sampled from a Beta.
    Beta(f32, f32),
}

/// Generate a Pareto or mixture of Paretos.
///
/// # Arguments
///
/// - `min: u32` - Plausible minimum number of insights to achieve AGI.
/// - `n_samps: usize` - Number of samples to take from the distribution.
/// - `mode: DistMode` - Which distribution to sample from.
pub fn pareto(min: u32, n_samps: usize, mode: DistMode) -> Vec<(f32, f32)> {
    // static allocation, prevent repeated work
    lazy_static! {
        static ref REQS: Vec<(f32, f32)> = linspace(0.005, 0.999, 500)
            .map(|x| {
                let max_cnt = CUM.last().unwrap().1 as f32;
                (x, max_cnt / x)
            })
            .collect();
    }

    // Xor shift random number generator (https://en.wikipedia.org/wiki/Xorshift) with known seed,
    // output should be deterministic up to floating point error.
    let mut rng = XorShiftRng::seed_from_u64(1);

    // parameterized Pareto.
    let lambda = |q: f32| -> f64 { -(1. - q).ln() as f64 };

    // pre-allocate vector
    let mut samples = Vec::with_capacity(n_samps);

    match mode {
        DistMode::Uniform => {
            let mut uni_rng = XorShiftRng::seed_from_u64(1);
            let dist = Uniform::from(0.001f32..1f32);

            for _ in 0..n_samps {
                samples.push(
                    dist.sample_iter(&mut uni_rng)
                        .map(|q| {
                            let dist = Exp::new(lambda(q)).unwrap();
                            dist.sample(&mut rng)
                        })
                        .map(|x| x.min(800.0) as f32) // prevent overflow
                        .map(|x| min as f32 * 2f32.powf(x))
                        .find(|&x| x > (CUM.last().unwrap().1 as f32)) // This is effectively a Bayesian update
                        .unwrap(),
                );
            }
        }
        DistMode::Q(q) => {
            let dist = Exp::new(lambda(q)).unwrap();

            for _ in 0..n_samps {
                samples.push(
                    dist.sample_iter(&mut rng)
                        .map(|x| x.min(800.0) as f32) // prevent overflow
                        .map(|x| min as f32 * 2f32.powf(x))
                        .find(|&x| x > (CUM.last().unwrap().1 as f32))
                        .unwrap(),
                );
            }
        }
        DistMode::Beta(alpha, beta) => {
            let mut beta_rng = XorShiftRng::seed_from_u64(1);

            let dist = Beta::new(alpha as f64, beta as f64).unwrap();
            for _ in 0..n_samps {
                samples.push(
                    dist.sample_iter(&mut beta_rng)
                        .map(|q| Exp::new(lambda(q as f32)).unwrap().sample(&mut rng))
                        .map(|x| x.min(800.0) as f32) // prevent overflow
                        .map(|x| min as f32 * 2f32.powf(x))
                        .find(|&x| x > (CUM.last().unwrap().1 as f32))
                        .unwrap(),
                );
            }
        }
    };

    REQS.iter()
        .copied()
        .map(|(p, r)| {
            let cnt: usize = samples.iter().copied().filter(|&x| x as f32 >= r).count();
            (p, cnt as f32 / n_samps as f32)
        })
        .collect()
}

/// Adds a point to the progress distribution, modifying other points as necessary to preserve
/// monotonicity.
///
/// # Arguments
///
/// - `pts: &mut Vec<(f32, f32)>` - The previous collection of points.
/// - `new_x: f32` - The x-coordinate of the point to be added, i.e., the proportion of required
/// insights that have been discovered.
/// - `new_y: f32` - The y-coordinate of the point to be added, i.e., the probability the the process
/// is no more than this much of the way done.
pub fn add_point(pts: &mut Vec<(f32, f32)>, new_x: f32, new_y: f32) -> Vec<(f32, f32)> {
    // The coordinate transformation is just an affine map.
    // This could be somewhat more robust and allow for dynamically scalable canvas
    // for drawing/displaying the distribution, but if would require slightly more
    // complicated state management.
    let xb = -1. / 9.;
    let xm = 1. / 450.;

    let ym = -1. / 420.;
    let yb = 15. / 14.;

    let new_x = xm * new_x + xb;
    let new_y = ym * new_y + yb;
    pts.push((new_x, new_y));

    pts.sort_unstable_by(|(k1, _), (k2, _)| k1.partial_cmp(k2).unwrap());

    // enforce monotonicity
    pts.iter()
        .copied()
        .map(|(x, y)| {
            if x < new_x {
                (x, y.min(new_y))
            } else if x > new_x {
                (x, y.max(new_y))
            } else {
                (x, new_y)
            }
        })
        .map(|(x, y)| (bound(x), bound(y)))
        .collect()
}

pub fn draw_dist(pts: &[(f32, f32)]) -> Option<()> {
    let pts = pts.iter().copied();

    // gracefully fail, and avoid the code gen bloat that happens with panics.
    let backend = CanvasBackend::new("progress_dist")?;

    let root = backend.into_drawing_area();

    root.fill(&White).ok()?;

    let font: FontDesc = ("Arial", 20.0).into();

    let mut chart = ChartBuilder::on(&root)
        .caption("Draw a progress distribution", font)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_ranged(0.0..1f32, 0.0..1f32)
        .ok()?;

    chart
        .configure_mesh()
        .x_desc("Proportion of required insights that have been discovered")
        .y_desc("Pr(no more than this much of the way done)")
        .draw()
        .ok()?;

    chart
        .draw_series(LineSeries::new(pts, &RGBColor(0, 136, 238)))
        .ok()?;

    Some(())
}
