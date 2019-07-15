use rand::distributions::Exp;

fn sample_pareto(min: f64, q: f64) {
    let lambda = -(1. - q).ln();
    let n_doublings = Exp.
}
