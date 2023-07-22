pub fn gear_resting_state((x, y): (f64, f64)) -> (f64, f64) {
    let x = if y < 0.5 && y > -0.5 {
        0.0
    } else if x > 0.9 {
        1.0
    } else if x < -0.9 {
        -1.0
    } else if x > 0.5 {
        0.75
    } else if x < -0.5 {
        -0.75
    } else {
        0.0
    };

    let y = if y > 0.9 {
        1.0
    } else if y < -0.9 {
        -1.0
    } else if y > 0.5 {
        0.75
    } else if y < -0.5 {
        -0.75
    } else {
        0.0
    };

    (x, y)
}
