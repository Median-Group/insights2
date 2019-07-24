use crate::{data::CUM, CurveFn};
use itertools_num::linspace;
use plotters::prelude::*;

/// Calculates the probability from the portion.
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

/// Draws the projected timeline.
///
/// # Arguments
///
/// - `max_year: i32` - Upper bound for how far to extend the projection.
/// - `pts: &[(f32, f32)]` - A progress distribution.
/// - `inv_curve: &Option<CurveFn>` - The inverse of the fit curve.
///
/// # Remark
///
/// The plot may not extend to `max_year` if large enough values cannot be obtained.
/// It is an upper bound, not a least upper bound.
pub(crate) fn draw_timeline(
    max_year: i32,
    pts: &[(f32, f32)],
    inv_curve: &Option<CurveFn>,
) -> Option<()> {
    let insights_cnt = CUM.last()?.1 as f32;

    let inv_curve = (*inv_curve).as_ref()?;

    let backend = CanvasBackend::new("timeline_plot")?;
    let root = backend.into_drawing_area();
    let font: FontDesc = ("Arial", 20.0).into();

    root.fill(&White).ok()?;

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

    let last_year = npts.last()?.0;

    let max_year = max_year.min(last_year);

    let mut chart = ChartBuilder::on(&root)
        .caption("Implied Timeline", font)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_ranged(2020..max_year, 0.01f32..1f32)
        .ok()?;

    chart
        .configure_mesh()
        .x_desc("Max Year")
        .y_desc("Probability")
        .draw()
        .ok()?;

    chart
        .draw_series(LineSeries::new(npts.into_iter(), &RGBColor(0, 136, 238)))
        .ok()?;

    Some(())
}
