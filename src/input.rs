use std::collections::HashMap;

#[derive(Debug)]
pub enum InputState {
    Inactive,
    Active,
    JustActive,
    JustInactive,
}

#[derive(Hash, Eq, PartialEq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Grab,
    Accelerate,
    Quit,
}

impl TryFrom<sdl2::keyboard::Keycode> for Action {
    type Error = String;

    fn try_from(value: sdl2::keyboard::Keycode) -> Result<Self, Self::Error> {
        match value {
            sdl2::keyboard::Keycode::W | sdl2::keyboard::Keycode::Up => Ok(Action::Up),
            sdl2::keyboard::Keycode::A | sdl2::keyboard::Keycode::Left => Ok(Action::Left),
            sdl2::keyboard::Keycode::S | sdl2::keyboard::Keycode::Down => Ok(Action::Down),
            sdl2::keyboard::Keycode::D | sdl2::keyboard::Keycode::Right => Ok(Action::Right),
            sdl2::keyboard::Keycode::Space => Ok(Action::Grab),
            sdl2::keyboard::Keycode::LShift => Ok(Action::Accelerate),
            key => Err(format!("unrecognized keycode: {key:#?}")),
        }
    }
}

impl TryFrom<sdl2::controller::Button> for Action {
    type Error = String;

    fn try_from(value: sdl2::controller::Button) -> Result<Self, Self::Error> {
        match value {
            sdl2::controller::Button::RightShoulder => Ok(Action::Grab),
            key => Err(format!("unrecognized keycode: {key:#?}")),
        }
    }
}

pub struct Input {
    action_map: HashMap<Action, InputState>,
    pub right_joystick: (f64, f64),
}

impl Input {
    pub fn new() -> Self {
        Self {
            action_map: HashMap::new(),
            right_joystick: (0.0, 0.0),
        }
    }

    pub fn update_right_joystick_from_raw_x(&mut self, value: i16) {
        let value = (f64::from(value) / f64::from(i16::MAX)) * 1.5;

        let value = if value > 1.0 {
            1.0
        } else if value < -1.0 {
            -1.0
        } else {
            value
        };

        self.right_joystick.0 = value;
    }

    pub fn update_right_joystick_from_raw_y(&mut self, value: i16) {
        let value = (f64::from(value) / f64::from(i16::MAX)) * 1.5;

        let value = if value > 1.0 {
            1.0
        } else if value < -1.0 {
            -1.0
        } else {
            value
        };

        self.right_joystick.1 = value;
    }

    pub fn get(&self, action: &Action) -> Option<&InputState> {
        self.action_map.get(action)
    }
    pub fn insert(&mut self, action: Action, state: InputState) -> Option<InputState> {
        self.action_map.insert(action, state)
    }
    pub fn action_active(&self, action: &Action) -> bool {
        matches!(
            self.get(action),
            Some(InputState::JustActive | InputState::Active)
        )
    }
    pub fn action_changed(&self, action: &Action) -> bool {
        matches!(
            self.get(action),
            Some(InputState::JustActive | InputState::JustInactive)
        )
    }

    pub fn action_tick(&mut self, action: Action) {
        let state = match self.get(&action) {
            Some(InputState::Inactive | InputState::JustInactive) | None => InputState::Inactive,
            Some(InputState::JustActive | InputState::Active) => InputState::Active,
        };
        self.insert(action, state);
    }
}
