use sdl2::keyboard::Keycode;

use crate::key_map::{change_key, key_changed, key_down, KeyMap};

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
    let a = key_down(&key_map, Keycode::A);
    let d = key_down(&key_map, Keycode::D);
    let target_x = match (a, d) {
        (true, true) | (false, false) => 0.0,
        (true, false) => -1.0,
        (false, true) => 1.0,
    };
    let w = key_down(&key_map, Keycode::W);
    let s = key_down(&key_map, Keycode::S);
    let target_y = match (w, s) {
        (true, true) | (false, false) => 0.0,
        (true, false) => -1.0,
        (false, true) => 1.0,
    };

    (target_x, target_y)
}

pub fn change_hand(key_map: &mut KeyMap) {
    change_key(key_map, Keycode::W);
    change_key(key_map, Keycode::A);
    change_key(key_map, Keycode::S);
    change_key(key_map, Keycode::D);
}

pub fn hand_changed(key_map: &KeyMap) -> bool {
    let w = key_changed(&key_map, Keycode::W);
    let a = key_changed(&key_map, Keycode::A);
    let s = key_changed(&key_map, Keycode::S);
    let d = key_changed(&key_map, Keycode::D);
    let space = key_changed(&key_map, Keycode::Space);

    w || a || s || d || space
}
