pub fn two_dimensional(alpha: f64, position: (f64, f64), target: (f64, f64)) -> (f64, f64) {
    (
        one_dimensional(alpha, position.0, target.0),
        one_dimensional(alpha, position.1, target.1),
    )
}

pub fn one_dimensional(alpha: f64, position: f64, target: f64) -> f64 {
    position + alpha * (target - position)
}
