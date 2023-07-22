pub fn gear_resting_target((x, y): (f64, f64)) -> (f64, f64) {
    let x = if y < 0.5 && y > -0.5 {
        0.0
    } else if x > 0.9 {
        1.0
    } else if x < -0.9 {
        -1.0
    } else if x > 0.25 || x < -0.25 {
        x
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

pub enum Gear {
    Neutral,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Rocket,
}

pub fn gear_state((x, y): (f64, f64)) -> Gear {
    if y >= -0.9 && y <= 0.9 {
        return Gear::Neutral;
    }
    if x <= 0.9 && x >= 0.25 {
        return Gear::Neutral;
    }
    if x >= -0.9 && x <= -0.25 {
        return Gear::Neutral;
    }

    if x < -0.9 {
        if y <= -0.9 {
            Gear::First
        } else {
            Gear::Second
        }
    } else if x < 0.25 {
        if y <= -0.9 {
            Gear::Third
        } else {
            Gear::Fourth
        }
    } else {
        if y <= -0.9 {
            Gear::Fifth
        } else {
            Gear::Rocket
        }
    }
}
