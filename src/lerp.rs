pub fn lerp2d(alpha: f64, position: (f64, f64), target: (f64, f64)) -> (f64, f64) {
    (
        lerp1d(alpha, position.0, target.0),
        lerp1d(alpha, position.1, target.1),
    )
}

pub fn lerp1d(alpha: f64, position: f64, target: f64) -> f64 {
    position + alpha * (target - position)
}
