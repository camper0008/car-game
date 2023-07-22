use std::collections::HashMap;

#[derive(Debug)]
pub enum KeyState {
    Up,
    Down,
    WasUp,
    WasDown,
}

#[derive(Hash, Eq, PartialEq)]
pub enum KeyCode {
    W,
    A,
    S,
    D,
    Space,
    Shift,
}

impl TryFrom<sdl2::keyboard::Keycode> for KeyCode {
    type Error = String;

    fn try_from(value: sdl2::keyboard::Keycode) -> Result<Self, Self::Error> {
        match value {
            sdl2::keyboard::Keycode::W | sdl2::keyboard::Keycode::Up => Ok(KeyCode::W),
            sdl2::keyboard::Keycode::A | sdl2::keyboard::Keycode::Left => Ok(KeyCode::A),
            sdl2::keyboard::Keycode::S | sdl2::keyboard::Keycode::Down => Ok(KeyCode::S),
            sdl2::keyboard::Keycode::D | sdl2::keyboard::Keycode::Right => Ok(KeyCode::D),
            sdl2::keyboard::Keycode::Space => Ok(KeyCode::Space),
            sdl2::keyboard::Keycode::LShift => Ok(KeyCode::Shift),
            key => Err(format!("unrecognized keycode: {key:#?}")),
        }
    }
}

impl TryFrom<sdl2::controller::Button> for KeyCode {
    type Error = String;

    fn try_from(value: sdl2::controller::Button) -> Result<Self, Self::Error> {
        match value {
            sdl2::controller::Button::RightShoulder => Ok(KeyCode::Space),
            key => Err(format!("unrecognized keycode: {key:#?}")),
        }
    }
}

pub type KeyMap = HashMap<KeyCode, KeyState>;

pub fn key_down(key_map: &KeyMap, key: &KeyCode) -> bool {
    matches!(key_map.get(&key), Some(KeyState::WasUp | KeyState::Down))
}

pub fn key_changed(key_map: &KeyMap, key: &KeyCode) -> bool {
    matches!(key_map.get(&key), Some(KeyState::WasUp | KeyState::WasDown))
}

pub fn change_key(key_map: &mut KeyMap, key: KeyCode) {
    let state = match key_map.get(&key) {
        Some(KeyState::Up | KeyState::WasDown) | None => KeyState::Up,
        Some(KeyState::WasUp | KeyState::Down) => KeyState::Down,
    };
    key_map.insert(key, state);
}
