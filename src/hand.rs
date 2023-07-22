use crate::key_map::{change_key, key_changed, key_down, KeyCode, KeyMap};

pub struct Hand {
    pub alpha: f64,
    pub offset: (f64, f64),
    pub target: (f64, f64),
}

impl Hand {
    pub fn gamepad_target(x: i16, y: i16) -> (f64, f64) {
        let x = (f64::from(x) / f64::from(i16::MAX)) * 1.5;
        let y = (f64::from(y) / f64::from(i16::MAX)) * 1.5;

        let x = if x > 1.0 {
            1.0
        } else if x < -1.0 {
            -1.0
        } else {
            x
        };
        let y = if y > 1.0 {
            1.0
        } else if y < -1.0 {
            -1.0
        } else {
            y
        };

        (x, y)
    }

    pub fn keyboard_target(key_map: &KeyMap) -> (f64, f64) {
        let a = key_down(key_map, &KeyCode::A);
        let d = key_down(key_map, &KeyCode::D);
        let target_x = match (a, d) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -1.0,
            (false, true) => 1.0,
        };
        let w = key_down(key_map, &KeyCode::W);
        let s = key_down(key_map, &KeyCode::S);
        let target_y = match (w, s) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -1.0,
            (false, true) => 1.0,
        };

        (target_x, target_y)
    }

    pub fn update_keys(key_map: &mut KeyMap) {
        change_key(key_map, KeyCode::W);
        change_key(key_map, KeyCode::A);
        change_key(key_map, KeyCode::S);
        change_key(key_map, KeyCode::D);
        change_key(key_map, KeyCode::Space);
    }

    pub fn has_changed(key_map: &KeyMap) -> bool {
        let w = key_changed(key_map, &KeyCode::W);
        let a = key_changed(key_map, &KeyCode::A);
        let s = key_changed(key_map, &KeyCode::S);
        let d = key_changed(key_map, &KeyCode::D);
        let space = key_changed(key_map, &KeyCode::Space);

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
