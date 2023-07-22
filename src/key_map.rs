use std::collections::HashMap;

use sdl2::keyboard::Keycode;

#[derive(Debug)]
pub enum KeyState {
    Up,
    Down,
    WasUp,
    WasDown,
}

pub type KeyMap = HashMap<Keycode, KeyState>;

pub fn key_down(key_map: &KeyMap, key: Keycode) -> bool {
    matches!(key_map.get(&key), Some(KeyState::WasUp | KeyState::Down))
}

pub fn key_changed(key_map: &KeyMap, key: Keycode) -> bool {
    matches!(key_map.get(&key), Some(KeyState::WasUp | KeyState::WasDown))
}

pub fn change_key(key_map: &mut KeyMap, key: Keycode) {
    let state = match key_map.get(&key) {
        Some(KeyState::Up | KeyState::WasDown) | None => KeyState::Up,
        Some(KeyState::WasUp | KeyState::Down) => KeyState::Down,
    };
    key_map.insert(key, state);
}
