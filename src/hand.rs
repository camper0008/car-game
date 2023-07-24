use crate::{gear_stick::Gear, input::Input, utils::clamp_f64};

pub struct Hand {
    pub smooth_factor: f64,
    pub offset: (f64, f64),
    pub target: (f64, f64),
}

impl Hand {
    pub fn target(input: &Input) -> (f64, f64) {
        input.hand
    }

    pub fn set_origin(&mut self, offset: (f64, f64)) {
        self.offset = offset;
    }
}

pub fn clamp_clutch_up(target: (f64, f64), old: (f64, f64), gear: &Gear) -> (f64, f64) {
    let (x_min, x_max) = match gear {
        Gear::Neutral => {
            if target.1 > -0.5 && target.1 < 0.5 {
                (-1.0, 1.0)
            } else if old.0 <= -0.925 {
                (-1.0, -0.925)
            } else if old.0 >= -0.25 && old.0 <= 0.25 {
                (-0.24, 0.24)
            } else if old.0 >= 0.925 {
                (0.925, 1.0)
            } else {
                (-1.0, 1.0)
            }
        }
        Gear::First | Gear::Second => (-1.0, -0.925),
        Gear::Third | Gear::Fourth => (-0.24, 0.24),
        Gear::Fifth | Gear::Rocket => (0.925, 1.0),
    };

    let (y_min, y_max) = match gear {
        Gear::Neutral => (-0.7, 0.7),
        Gear::First | Gear::Third | Gear::Fifth => (-1.0, -0.95),
        Gear::Second | Gear::Fourth | Gear::Rocket => (0.95, 1.0),
    };

    let x = clamp_f64(target.0, x_min, x_max);
    let y = clamp_f64(target.1, y_min, y_max);

    (x, y)
}

pub fn clamp_clutch_down(target: (f64, f64), old: (f64, f64)) -> (f64, f64) {
    if target.1 > -0.5 && target.1 < 0.5 {
        return target;
    }

    if (target.0 - old.0).abs() < 0.5 {
        return target;
    }

    let target_x = if old.0 <= -0.5 {
        -0.51
    } else if old.0 >= 0.5 {
        0.51
    } else if target.0 >= 0.5 {
        0.49
    } else if target.0 <= -0.5 {
        -0.49
    } else {
        return target;
    };

    (target_x, target.1)
}
