use sdl2::keyboard::Keycode;

use crate::key_map::{change_key, key_changed, key_down, KeyMap};

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
    let a = key_changed(&key_map, Keycode::A);
    let d = key_changed(&key_map, Keycode::D);
    let w = key_changed(&key_map, Keycode::W);
    let s = key_changed(&key_map, Keycode::S);

    //println!("w: {w}, a: {a}, s: {s}, d: {d}");
    w || a || s || d
}
