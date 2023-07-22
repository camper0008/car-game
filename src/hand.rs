use crate::key_map::{change_key, key_changed, key_down, KeyCode, KeyMap};

pub fn clamp_hand(target: (f64, f64), old: (f64, f64)) -> (f64, f64) {
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

pub fn hand_target(key_map: &KeyMap) -> (f64, f64) {
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

pub fn change_hand(key_map: &mut KeyMap) {
    change_key(key_map, KeyCode::W);
    change_key(key_map, KeyCode::A);
    change_key(key_map, KeyCode::S);
    change_key(key_map, KeyCode::D);
    change_key(key_map, KeyCode::Space);
}

pub fn hand_changed(key_map: &KeyMap) -> bool {
    let w = key_changed(key_map, &KeyCode::W);
    let a = key_changed(key_map, &KeyCode::A);
    let s = key_changed(key_map, &KeyCode::S);
    let d = key_changed(key_map, &KeyCode::D);
    let space = key_changed(key_map, &KeyCode::Space);

    w || a || s || d || space
}
