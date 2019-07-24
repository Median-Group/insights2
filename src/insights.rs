use crate::{util::fit_sigmoid, CurveFn, RegMode};
use linreg::linear_regression_of;
use plotters::prelude::*;

/// Draw the cumulative distribution of insights over a specified time period, accompanied by a
/// best fit curve.
///
/// # Arguments
///
/// - `min_year: i16` - The minium year to display.
/// - `max_year: i16` - The maximum year to display.
/// - `sub_cum: i16` - The subset of the cumulative distribution of insights in the time interval.
/// - `model_curve: &Option<CurveFn>` - A function that computes the curve fit to the insights
/// data.
/// - `curve_repr` - A string representing the equation of the curve.
pub fn draw_insights(
    min_year: i16,
    max_year: i16,
    sub_cum: &[(i16, u8)],
    model_curve: &Option<CurveFn>,
    curve_repr: String,
) -> Option<()> {
    if sub_cum.len() < 5 {
        return None;
    }

    // Lower and upper bounds for insight counts in the year range,
    // needed to setup the chart
    let insights_lb: f32 = sub_cum[0].1 as f32;
    let mut insights_ub: f32 = sub_cum.last()?.1 as f32;

    // The control flow here is a bit complicated. We need to know how large the fit curve gets
    // in order to size the plot properly, but we also want to be able to show just the data if
    // `model_curve` is `None`.
    let model_curve = match model_curve {
        Some(mc) => {
            insights_ub = insights_ub.max(mc(sub_cum.last()?.0 as i32));
            Some(mc)
        }
        None => None,
    };

    let backend = CanvasBackend::new("insights_plot")?;
    let root = backend.into_drawing_area();
    let font: FontDesc = ("Arial", 20.0).into();

    root.fill(&White).ok()?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Timeline of AI progress", font)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_ranged(
            min_year as i32..max_year as i32,
            (0.9 * insights_lb)..(1.1 * insights_ub),
        )
        .ok()?;

    chart
        .configure_mesh()
        .y_label_formatter(&|x| (*x as i32).to_string())
        .x_labels(10)
        .y_labels(0)
        .x_desc("Year")
        .y_desc("Cumulative number of insights")
        .draw()
        .ok()?;

    // data
    chart
        .draw_series(LineSeries::new(
            sub_cum
                .iter()
                .copied()
                .map(|(year, count)| (year as i32, count as f32)),
            &RGBColor(0, 136, 238),
        ))
        .ok()?
        .label("Insights".to_string())
        .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &RGBColor(0, 136, 238)));

    // fit
    if let Some(curve) = model_curve {
        chart
            .draw_series(LineSeries::new(
                (sub_cum[0].0..=sub_cum.last()?.0)
                    .map(|y| (y, curve(y as i32)))
                    .map(|(year, pred)| (year as i32, pred))
                    .filter(|(_, pred)| *pred > 0.),
                &RGBColor(238, 85, 0),
            ))
            .ok()?
            .label(curve_repr)
            .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &RGBColor(238, 85, 0)));
    }

    chart
        .configure_series_labels()
        .border_style(&Black)
        .draw()
        .ok()?;

    Some(())
}

/// Fits
pub fn reg(mode: &RegMode, sub_cum: &[(i16, u8)]) -> Option<(String, CurveFn, CurveFn)> {
    use RegMode::*;
    match mode {
        Linear => {
            let (m, b): (f32, f32) = linear_regression_of(sub_cum)?;

            let func = Box::new(move |y| m * y as f32 + b);
            let inv_func = Box::new(move |cnt| (cnt as f32 - b) / m);

            let repr = format!("y = {m:.2}x + {b:.2}", m = m, b = b);
            Some((repr, func, inv_func))
        }
        Exponential => {
            // Linear regression in log-space to fit an exponential. For things that aren't very
            // close to clean exponentials, this will over-weight smaller values relative to larger
            // values, but fixing this would require adjusting the weight matrix.
            let log_cum: Vec<(f32, f32)> = sub_cum
                .iter()
                .map(|(year, cnt)| (*year as f32, (*cnt as f32).ln()))
                .collect();

            let (rate, constant): (f32, f32) = linear_regression_of(&log_cum)?;

            let func = Box::new(move |y| (rate * y as f32).exp() * (constant).exp());
            let inv_func = Box::new(move |cnt| (cnt as f32 / constant.exp()).ln() / rate);

            let repr = format!(
                "y = {constant:.3E} e^({rate:.5} * x)",
                constant = constant.exp(),
                rate = rate
            );
            Some((repr, func, inv_func))
        }
        Sigmoidal => {
            let pts: Vec<(f32, f32)> = sub_cum
                .iter()
                .copied()
                .map(|(y, cnt)| (y as f32, cnt as f32))
                .collect();

            let (m, k, b) = fit_sigmoid(&pts)?;

            let func = Box::new(move |y| m / (1. + (-k * (y as f32 - b)).exp()));
            let inv_func = Box::new(move |cnt| ((b * k) - (m / cnt as f32 - 1.).ln()) / k);

            let repr = format!(
                "y = {m:.1} / (1 + e^(-{k:.2E} * (x - {b:.1})))",
                m = m,
                k = k,
                b = b
            );
            Some((repr, func, inv_func))
        }
    }
}
