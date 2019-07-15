use differential_evolution2::self_adaptive_de;

pub fn sigmoid(m: f32, k: f32, b: f32, x: f32) -> f32 {
    m / (1. + (-k * (x - b)).exp())
}

pub fn fit_sigmoid(pts: &[(f32, f32)]) -> Option<(f32, f32, f32)> {
    let cost = |pos: &[f32]| -> f32 {
        let (m, k, b): (f32, f32, f32) = (pos[0], pos[1], pos[2]);
        pts.iter()
            .copied()
            .map(|(y, cnt)| (sigmoid(m, k, b, y) - cnt).powi(2))
            .sum::<f32>()
            / (pts.len() as f32).powi(2)
    };

    let initial: Vec<(f32, f32)> = vec![(300., 1200.), (0.01, 0.03), (-1850., 2200.)];

    let mut de = self_adaptive_de(initial, cost);

    let n_iters = 20_000;

    de.iter().nth(n_iters);
    let (cost, params) = de.best().unwrap();

    if *cost < 1.0 {
        Some((params[0], params[1], params[2]))
    } else {
        None
    }
}

//#[macro_export]
//macro_rules! log {
//    { $($expr:expr),* $(,)* } => {
//        {
//            let mut text = String::new();
//            $(
//                text += &$expr.to_string();
//                text += " ";
//            )*
//            web_sys::console::log_1(&text.into());
//        }
//     };
//}
