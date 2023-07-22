use crate::input::{Action, Input};

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
    }

    pub fn has_changed(input: &Input) -> bool {
        let w = input.action_changed(&Action::Up);
        let a = input.action_changed(&Action::Left);
        let s = input.action_changed(&Action::Down);
        let d = input.action_changed(&Action::Right);
        let space = input.action_changed(&Action::Grab);

        w || a || s || d || space
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

pub fn clamp(target: (f64, f64), old: (f64, f64)) -> (f64, f64) {
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
