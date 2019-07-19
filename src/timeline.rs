use crate::data::CUM;
use itertools_num::linspace;
use plotters::prelude::*;

fn prob_from_portion(mut portion: f32, pts: &[(f32, f32)]) -> Option<f32> {
    if portion <= 0.0 {
        portion = 0.00;
    } else if portion >= 1.0 {
        portion = 1.0;
    }

    for i in 0..pts.len() - 1 {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[i + 1];

        if (x1..=x2).contains(&portion) {
            return Some(y1 + (portion - x1) / (x2 - x1) * (y2 - y1));
        }
    }

    None
}

pub fn draw_timeline(max_year: i32, pts: &[(f32, f32)], inv_curve: &Option<Box<Fn(i32) -> f32>>) {
    let insights_cnt = CUM.last().unwrap().1 as f32;

    let inv_curve = match inv_curve {
        Some(ic) => ic,
        None => return,
    };

    let backend = CanvasBackend::new("timeline_plot").expect("Can't access backend");
    let root = backend.into_drawing_area();
    let font: FontDesc = ("Arial", 20.0).into();

    if root.fill(&White).is_err() {
        return;
    }

    let npts: Vec<(i32, f32)> = linspace(0.01, 0.999, 10_000)
        .map(|portion| {
            (
                inv_curve((insights_cnt / portion) as i32) as i32,
                1. - prob_from_portion(portion, pts).unwrap() as f32,
            )
        })
        .filter(|(y, _)| *y >= 2020)
        .rev()
        .collect();

    let last_year = match npts.last() {
        Some(pt) => pt.0,
        None => return,
    };

    let max_year = max_year.min(last_year);

    let mut chart = match ChartBuilder::on(&root)
        .caption("Implied Timeline", font)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_ranged(2020..max_year, 0.01f32..1f32)
    {
        Ok(c) => c,
        Err(_) => return,
    };

    chart
        .configure_mesh()
        .x_desc("Max Year")
        .y_desc("Probability")
        .draw()
        .unwrap();

    // extend the plot to the end of the range
    //npts.push((max_year, last_p));

    chart
        .draw_series(LineSeries::new(npts.into_iter(), &RGBColor(0, 136, 238)))
        .unwrap();
}
