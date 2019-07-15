use crate::data::CUM;
use itertools_num::linspace;
use lazy_static::*;
use plotters::prelude::*;
use rand::prelude::*;
use rand_distr::{Beta, Exp, Uniform};
use rand_xorshift::XorShiftRng;

fn bound(x: f32) -> f32 {
    if x < 0. {
        0.
    } else if x > 1. {
        1.
    } else {
        x
    }
}

pub enum DistMode {
    Uniform,
    Q(f32),
    Beta(f32, f32),
}

pub fn pareto(min: u32, n_samps: usize, mode: DistMode) -> Vec<(f32, f32)> {
    lazy_static! {
        static ref REQS: Vec<(f32, f32)> = linspace(0.005, 0.999, 500)
            .map(|x| {
                let max_cnt = CUM.last().unwrap().1 as f32;
                (x, max_cnt / x)
            })
            .collect();
    }

    let mut rng = XorShiftRng::seed_from_u64(1);

    let lambda = |q: f32| -> f64 { -(1. - q).ln() as f64 };

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
                        .find(|&x| x > (CUM.last().unwrap().1 as f32))
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

pub fn add_point(pts: &mut Vec<(f32, f32)>, new_x: f32, new_y: f32) -> Vec<(f32, f32)> {
    // the coordinate transformation is just an affine map
    let xb = -1. / 9.;
    let xm = 1. / 450.;

    let ym = -1. / 420.;
    let yb = 15. / 14.;

    let new_x = xm * new_x + xb;
    let new_y = ym * new_y + yb;
    pts.push((new_x, new_y));

    pts.sort_unstable_by(|(k1, _), (k2, _)| k1.partial_cmp(k2).unwrap());

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

pub fn draw_dist(pts: &[(f32, f32)]) {
    let pts = pts.iter().copied();

    let backend = CanvasBackend::new("progress_dist").expect("Can't access backend");

    let root = backend.into_drawing_area();

    if root.fill(&White).is_err() {
        return;
    }

    let font: FontDesc = ("Arial", 20.0).into();

    let mut chart = match ChartBuilder::on(&root)
        .caption("Draw a progress distribution", font)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_ranged(0.0..1f32, 0.0..1f32)
    {
        Ok(c) => c,
        Err(_) => return,
    };

    chart
        .configure_mesh()
        .x_desc("Proportion of required insights that have been discovered")
        .y_desc("Pr(no more than this much of the way done)")
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(pts, &RGBColor(0, 136, 238)))
        .unwrap();
}
