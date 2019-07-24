use crate::{util::fit_sigmoid, CurveFn};
use linreg::linear_regression_of;
use plotters::prelude::*;

pub fn draw_insights(
    min_year: i16,
    max_year: i16,
    cum: &[(i16, u8)],
    model_curve: &Option<CurveFn>,
    curve_repr: String,
    //mode: String,
) {
    if cum.len() < 5 {
        return;
    }

    // Lower and upper bounds for insight counts in the year range,
    // needed to setup the chart
    let insights_lb: f32 = cum[0].1 as f32;
    let mut insights_ub: f32 = cum.last().unwrap().1 as f32;

    // get predicted values
    let model_curve = match model_curve {
        Some(mc) => {
            insights_ub = insights_ub.max(mc(cum.last().unwrap().0 as i32));
            Some(mc)
        }
        None => None,
    };
    let backend = CanvasBackend::new("insights_plot").expect("Can't access backend");
    let root = backend.into_drawing_area();
    let font: FontDesc = ("Arial", 20.0).into();

    if root.fill(&White).is_err() {
        return;
    }

    let mut chart = match ChartBuilder::on(&root)
        .caption("Timeline of AI progress", font)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_ranged(
            min_year as i32..max_year as i32,
            (0.9 * insights_lb)..(1.1 * insights_ub),
        ) {
        Ok(c) => c,
        Err(_) => return,
    };

    chart
        .configure_mesh()
        .y_label_formatter(&|x| (*x as i32).to_string())
        .x_labels(10)
        .y_labels(0)
        .x_desc("Year")
        .y_desc("Cumulative number of insights")
        .draw()
        .unwrap();

    // data
    chart
        .draw_series(LineSeries::new(
            cum.iter()
                .copied()
                .map(|(year, count)| (year as i32, count as f32)),
            &RGBColor(0, 136, 238),
        ))
        .unwrap()
        .label("Insights".to_string())
        .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &RGBColor(0, 136, 238)));

    // fit
    if let Some(curve) = model_curve {
        chart
            .draw_series(LineSeries::new(
                (cum[0].0..=cum.last().unwrap().0)
                    .map(|y| (y, curve(y as i32)))
                    .map(|(year, pred)| (year as i32, pred))
                    .filter(|(_, pred)| *pred > 0.),
                &RGBColor(238, 85, 0),
            ))
            .unwrap()
            .label(curve_repr)
            .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &RGBColor(238, 85, 0)));
    }

    chart
        .configure_series_labels()
        .border_style(&Black)
        .draw()
        .unwrap();
}

pub fn reg(mode: &str, cum: &[(i16, u8)]) -> Option<(String, CurveFn, CurveFn)> {
    match mode {
        "Linear" => {
            let (m, b): (f32, f32) = linear_regression_of(&cum)?;

            let func = Box::new(move |y| m * y as f32 + b);
            let inv_func = Box::new(move |cnt| (cnt as f32 - b) / m);

            let repr = format!("y = {m:.2}x + {b:.2}", m = m, b = b);
            Some((repr, func, inv_func))
        }
        "Exponential" => {
            let log_cum: Vec<(f32, f32)> = cum
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
        "Sigmoidal" => {
            let pts: Vec<(f32, f32)> = cum
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
        _ => unimplemented!(),
    }
}
