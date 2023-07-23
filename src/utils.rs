pub fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn lerp_2d(alpha: f64, position: (f64, f64), target: (f64, f64)) -> (f64, f64) {
    (
        lerp_1d(alpha, position.0, target.0),
        lerp_1d(alpha, position.1, target.1),
    )
}

pub fn lerp_1d(alpha: f64, position: f64, target: f64) -> f64 {
    position + alpha * (target - position)
}
