use crate::{
    gear::Speed,
    input::{Action, Input},
};

pub struct Hand {
    pub alpha: f64,
    pub offset: (f64, f64),
    pub target: (f64, f64),
}

impl Hand {
    pub fn gamepad_target(input: &Input) -> (f64, f64) {
        input.right_joystick
    }
    pub fn keyboard_target(input: &Input) -> (f64, f64) {
        let a = input.action_active(&Action::Left);
        let d = input.action_active(&Action::Right);
        let target_x = match (a, d) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -1.0,
            (false, true) => 1.0,
        };
        let w = input.action_active(&Action::Up);
        let s = input.action_active(&Action::Down);
        let target_y = match (w, s) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -1.0,
            (false, true) => 1.0,
        };

        (target_x, target_y)
    }

    pub fn action_tick(input: &mut Input) {
        input.action_tick(Action::Up);
        input.action_tick(Action::Left);
        input.action_tick(Action::Down);
        input.action_tick(Action::Right);
        input.action_tick(Action::Grab);
        input.action_tick(Action::Clutch);
    }

    pub fn has_changed(input: &Input) -> bool {
        let up = input.action_changed(&Action::Up);
        let left = input.action_changed(&Action::Left);
        let down = input.action_changed(&Action::Down);
        let right = input.action_changed(&Action::Right);
        let grab = input.action_changed(&Action::Grab);
        let clutch = input.action_changed(&Action::Clutch);

        up || left || down || right || grab || clutch
    }

    pub fn reset(&mut self, alpha: f64, offset: (f64, f64)) {
        self.alpha = alpha;
        self.offset = offset;
    }

    pub fn tick(&mut self, tick: f64) {
        self.alpha += tick;
        if self.alpha > 1.0 {
            self.alpha = 1.0;
        }
    }
}

fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn clamp_clutch_up(target: (f64, f64), speed: Speed) -> (f64, f64) {
    let (x_min, x_max) = match speed {
        Speed::Neutral => (-1.0, 1.0),
        Speed::First | Speed::Second => (-1.0, -0.925),
        Speed::Third | Speed::Fourth => (-0.24, 0.24),
        Speed::Fifth | Speed::Rocket => (0.925, 1.0),
    };

    let (y_min, y_max) = match speed {
        Speed::Neutral => (-0.7, 0.7),
        Speed::First | Speed::Third | Speed::Fifth => (-1.0, -0.95),
        Speed::Second | Speed::Fourth | Speed::Rocket => (0.95, 1.0),
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

    let target_x = if old.0 < -0.5 {
        -0.51
    } else if old.0 > 0.5 {
        0.51
    } else if target.0 > 0.5 {
        0.5
    } else if target.0 < -0.5 {
        -0.5
    } else {
        return target;
    };

    (target_x, target.1)
}
