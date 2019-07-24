#![allow(clippy::cast_lossless)]

use data::CUM;
use wasm_bindgen::prelude::*;

mod data;
mod dist;
mod insights;
mod timeline;
mod util;

type CurveFn = Box<dyn Fn(i32) -> f32>;

#[wasm_bindgen]
pub struct State {
    prog_points: Vec<(f32, f32)>,
    year_min: i16,
    year_max: i16,
    last: i32,
    mode: String,
    num_samples: u16,
    model_curve: Option<CurveFn>,
    curve_repr: Option<String>,
    inv_curve: Option<CurveFn>,
    sub_cum: Vec<(i16, u8)>,
}

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
        let mode = "Linear";
        let sub_cum = make_sub_cum(year_min, year_max);
        let (curve_repr, curve, inv_curve) = insights::reg(mode, &sub_cum).unwrap();

        State {
            prog_points: vec![(0., 0.), (1., 1.)],
            year_min,
            year_max,
            num_samples: 5000,
            last: 2200,
            mode: mode.into(),
            curve_repr: Some(curve_repr),
            model_curve: Some(curve),
            inv_curve: Some(inv_curve),
            sub_cum,
        }
    }

    pub fn set_year_range(&mut self, year_min: i16, year_max: i16) {
        self.year_min = year_min;
        self.year_max = year_max;

        self.sub_cum = make_sub_cum(year_min, year_max);
        self.set_curve();
    }

    pub fn set_mode(&mut self, mode: String) {
        self.mode = mode;
        self.set_curve();
    }

    pub fn set_last(&mut self, last: i32) {
        self.last = last;
    }

    pub fn add_point(&mut self, new_x: f32, new_y: f32) {
        self.prog_points = dist::add_point(&mut self.prog_points, new_x, new_y);
    }

    pub fn reset_progress(&mut self) {
        self.prog_points = vec![(0., 0.), (1., 1.)];
        self.draw_dist();
        self.draw_timeline();
    }

    pub fn draw_dist(&self) {
        dist::draw_dist(&self.prog_points);
    }

    pub fn set_num_samples(&mut self, n: u16) {
        self.num_samples = n;
    }

    pub fn set_pareto(&mut self, min: u32, q: f32) {
        let mode = dist::DistMode::Q(q);
        let pts = dist::pareto(min, self.num_samples as usize, mode);
        self.prog_points = pts;
    }

    pub fn set_pareto_uniform(&mut self, min: u32) {
        let pts = dist::pareto(min, self.num_samples as usize, dist::DistMode::Uniform);
        self.prog_points = pts;
    }

    pub fn set_pareto_beta(&mut self, min: u32, alpha: f32, beta: f32) {
        let mode = dist::DistMode::Beta(alpha, beta);
        let pts = dist::pareto(min, self.num_samples as usize, mode);
        self.prog_points = pts;
    }

    pub fn draw_insights(&mut self) {
        insights::draw_insights(
            self.year_min,
            self.year_max,
            &self.sub_cum,
            &self.model_curve,
            self.curve_repr.clone().unwrap_or_else(|| "".into()),
        );
    }

    pub fn draw_timeline(&self) {
        timeline::draw_timeline(self.last, &self.prog_points, &self.inv_curve);
    }
}

impl State {
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
