#![allow(clippy::cast_lossless)]

use data::CUM;
use wasm_bindgen::prelude::*;

mod data;
mod dist;
mod insights;
mod timeline;
mod util;

/// Type-alias for the curves fit to the insights data.
type CurveFn = Box<dyn Fn(i32) -> f32>;

/// Which model to fit the insights data to.
pub enum RegMode {
    Linear,
    Exponential,
    Sigmoidal,
}

#[wasm_bindgen]
/// The state of the insights demo.
pub struct State {
    /// The points the specify the prior distribution.
    prog_points: Vec<(f32, f32)>,
    /// The minimum year to display and fit the curve to.
    year_min: i16,
    /// The maximum year to display and fit the curve to.
    year_max: i16,
    /// An upper bound for how far the timeline projection goes.
    last: i32,
    /// Which model to fit the insights data to.
    mode: RegMode,
    /// The number of samples to take from the prior distribution.
    num_samples: u16,
    /// The fit curve.
    model_curve: Option<CurveFn>,
    /// A string representation of the equation describing the fit curve.
    curve_repr: Option<String>,
    /// The inverse of the fit curve (used for the projection).
    inv_curve: Option<CurveFn>,
    /// The subset of the cumulative distribution of insights under consideration.
    sub_cum: Vec<(i16, u8)>,
}

/// Takes the subset of the cumulative distribution of insights in a given time interval.
///
/// # Arguments
///
/// - `year_min: i16` - The lower bound of the interval.
/// - `year_max: i16` - The upper bound of the interval.
fn make_sub_cum(year_min: i16, year_max: i16) -> Vec<(i16, u8)> {
    CUM.iter()
        .skip_while(|(y, _)| *y < year_min)
        .take_while(|(y, _)| *y <= year_max)
        .copied()
        .collect()
}

#[wasm_bindgen]
impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> State {
        let year_min = 1945;
        let year_max = 2016;
        let mode = RegMode::Linear;
        let sub_cum = make_sub_cum(year_min, year_max);
        let (curve_repr, curve, inv_curve) = insights::reg(&mode, &sub_cum).unwrap();

        State {
            prog_points: vec![(0., 0.), (1., 1.)],
            year_min,
            year_max,
            num_samples: 5000,
            last: 2200,
            mode,
            curve_repr: Some(curve_repr),
            model_curve: Some(curve),
            inv_curve: Some(inv_curve),
            sub_cum,
        }
    }

    /// Sets time interval for insights.
    ///
    /// # Arguments
    ///
    /// - `year_min: i16` - The lower bound of the interval.
    /// - `year_max: i16` - The upper bound of the interval.
    pub fn set_year_range(&mut self, year_min: i16, year_max: i16) {
        self.year_min = year_min;
        self.year_max = year_max;

        self.sub_cum = make_sub_cum(year_min, year_max);
        self.set_curve();
    }

    /// Sets the regression mode for curve fitting.
    ///
    /// # Argument
    ///
    /// - `mode: &str` - String representation of mode.
    pub fn set_mode(&mut self, mode: &str) {
        self.mode = match mode {
            "Linear" => RegMode::Linear,
            "Exponential" => RegMode::Exponential,
            "Sigmoidal" => RegMode::Sigmoidal,
            _ => unimplemented!(),
        };
        self.set_curve();
    }

    /// Sets upper bound for the last year in the projection.
    ///
    /// # Argument
    ///
    /// - `last: i32` - Upper bound for the last year in the projection.
    pub fn set_last(&mut self, last: i32) {
        self.last = last;
    }

    /// Adds a point to the progress distribution.
    ///
    /// # Arguments
    ///
    /// - `new_x: f32` - The x-coordinate of the point to be added, i.e., the proportion of required
    /// insights that have been discovered.
    /// - `new_y: f32` - The y-coordinate of the point to be added, i.e., the probability the the process
    /// is no more than this much of the way done.
    pub fn add_point(&mut self, new_x: f32, new_y: f32) {
        self.prog_points = dist::add_point(&mut self.prog_points, new_x, new_y);
    }

    /// Resets progress distribution to initial values.
    pub fn reset_progress(&mut self) {
        self.prog_points = vec![(0., 0.), (1., 1.)];
        self.draw_dist();
        self.draw_timeline();
    }

    /// Draws the progress distribution to a canvas.
    pub fn draw_dist(&self) {
        dist::draw_dist(&self.prog_points);
    }

    /// Sets the number of samples to take from the progression distribution.
    pub fn set_num_samples(&mut self, n: u16) {
        self.num_samples = n;
    }

    /// Sets the distribution to a Pareto with the chosen parameters.
    ///
    /// # Arguments
    /// - `min: u32` - The minimum plausible number of insights.
    /// - `q: f32` - The `q` parameter for the Pareto.
    pub fn set_pareto(&mut self, min: u32, q: f32) {
        let pts = dist::pareto(min, self.num_samples as usize, dist::DistMode::Q(q));
        self.prog_points = pts;
    }

    /// Sets the distribution to a mixture of Paretos where the `q` parameters are sampled from a
    /// uniformly from [0.001, 1).
    ///
    /// # Arguments
    /// - `min: u32` - The minimum plausible number of insights.
    pub fn set_pareto_uniform(&mut self, min: u32) {
        let pts = dist::pareto(min, self.num_samples as usize, dist::DistMode::Uniform);
        self.prog_points = pts;
    }

    /// Sets the distribution to a mixture of Paretos where the `q` parameters are sampled from a
    /// Beta distribution parameterized by `alpha` and `beta`.
    pub fn set_pareto_beta(&mut self, min: u32, alpha: f32, beta: f32) {
        let mode = dist::DistMode::Beta(alpha, beta);
        let pts = dist::pareto(min, self.num_samples as usize, mode);
        self.prog_points = pts;
    }

    /// Draws the insights plot.
    pub fn draw_insights(&mut self) {
        insights::draw_insights(
            self.year_min,
            self.year_max,
            &self.sub_cum,
            &self.model_curve,
            self.curve_repr.clone().unwrap_or_else(|| "".into()),
        );
    }

    /// Draws the tinmeline plot.
    pub fn draw_timeline(&self) {
        timeline::draw_timeline(self.last, &self.prog_points, &self.inv_curve);
    }
}

impl State {
    /// Updates the curve.
    fn set_curve(&mut self) {
        match insights::reg(&self.mode, &self.sub_cum) {
            Some((repr, curve, inv)) => {
                self.curve_repr = Some(repr);
                self.model_curve = Some(curve);
                self.inv_curve = Some(inv);
            }
            None => {
                self.model_curve = None;
                self.inv_curve = None;
            }
        }
    }
}
