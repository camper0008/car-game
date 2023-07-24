use std::collections::HashMap;

use sdl2::controller::GameController;

use crate::utils::clamp_f64;

#[derive(Debug)]
pub enum ActionState {
    Inactive,
    Active,
    JustActive,
    JustInactive,
}

#[derive(Hash, Eq, PartialEq)]
pub enum Action {
    Grab,
    Accelerate,
    Brake,
    Clutch,
    Quit,
}

impl TryFrom<sdl2::keyboard::Keycode> for Action {
    type Error = String;

    fn try_from(value: sdl2::keyboard::Keycode) -> Result<Self, Self::Error> {
        match value {
            sdl2::keyboard::Keycode::W | sdl2::keyboard::Keycode::Up => Ok(Action::Accelerate),
            sdl2::keyboard::Keycode::S | sdl2::keyboard::Keycode::Down => Ok(Action::Brake),
            sdl2::keyboard::Keycode::Space => Ok(Action::Grab),
            sdl2::keyboard::Keycode::LShift => Ok(Action::Clutch),
            key => Err(format!("unrecognized keycode: {key:#?}")),
        }
    }
}

impl TryFrom<sdl2::controller::Button> for Action {
    type Error = String;

    fn try_from(value: sdl2::controller::Button) -> Result<Self, Self::Error> {
        match value {
            sdl2::controller::Button::RightShoulder => Ok(Action::Grab),
            sdl2::controller::Button::LeftShoulder => Ok(Action::Clutch),
            key => Err(format!("unrecognized keycode: {key:#?}")),
        }
    }
}

impl TryFrom<sdl2::mouse::MouseButton> for Action {
    type Error = String;

    fn try_from(value: sdl2::mouse::MouseButton) -> Result<Self, Self::Error> {
        match value {
            sdl2::mouse::MouseButton::Left => Ok(Action::Grab),
            key => Err(format!("unrecognized keycode: {key:#?}")),
        }
    }
}

pub struct Input {
    action_map: HashMap<Action, ActionState>,
    mouse_sensitivity: f64,
    pub brake_alpha: f64,
    pub speeder_alpha: f64,
    pub hand: (f64, f64),
    pub active_controller: Option<GameController>,
}

impl Input {
    pub fn with_sensitivity(mouse_sensitivity: f64) -> Self {
        Self {
            action_map: HashMap::new(),
            hand: (0.0, 0.0),
            mouse_sensitivity,
            brake_alpha: 1.0,
            speeder_alpha: 1.0,
            active_controller: None,
        }
    }

    pub fn shake_controller(&mut self) {
        if let Some(controller) = &mut self.active_controller {
            if let Err(err) = controller.set_rumble(u16::MAX, u16::MAX, 500) {
                log::warn!("unable to rumble: {err}");
            }
        }
    }

    pub fn update_hand_relatively(&mut self, x: i32, y: i32) {
        let reduced_x = self.hand.0 + f64::from(x) / self.mouse_sensitivity;
        let reduced_x = clamp_f64(reduced_x, -1.0, 1.0);
        let reduced_y = self.hand.1 + f64::from(y) / self.mouse_sensitivity;
        let reduced_y = clamp_f64(reduced_y, -1.0, 1.0);

        self.hand = (reduced_x, reduced_y);
    }

    pub fn update_hand_from_raw_x(&mut self, value: i16) {
        let value = (f64::from(value) / f64::from(i16::MAX)) * 1.5;

        let value = if value > 1.0 {
            1.0
        } else if value < -1.0 {
            -1.0
        } else {
            value
        };

        self.hand.0 = value;
    }

    pub fn update_hand_from_raw_y(&mut self, value: i16) {
        let value = (f64::from(value) / f64::from(i16::MAX)) * 1.5;

        let value = if value > 1.0 {
            1.0
        } else if value < -1.0 {
            -1.0
        } else {
            value
        };

        self.hand.1 = value;
    }

    pub fn get(&self, action: &Action) -> Option<&ActionState> {
        self.action_map.get(action)
    }
    pub fn insert(&mut self, action: Action, state: ActionState) -> Option<ActionState> {
        self.action_map.insert(action, state)
    }
    pub fn action_active(&self, action: &Action) -> bool {
        matches!(
            self.get(action),
            Some(ActionState::JustActive | ActionState::Active)
        )
    }
    pub fn action_changed(&self, action: &Action) -> bool {
        matches!(
            self.get(action),
            Some(ActionState::JustActive | ActionState::JustInactive)
        )
    }

    pub fn action_tick(&mut self, action: Action) {
        let state = match self.get(&action) {
            Some(ActionState::Inactive | ActionState::JustInactive) | None => ActionState::Inactive,
            Some(ActionState::JustActive | ActionState::Active) => ActionState::Active,
        };
        self.insert(action, state);
    }

    pub fn key_down<A: TryInto<Action> + std::fmt::Debug + Copy>(&mut self, action: A) {
        let Ok(action) = action.try_into() else {
        log::debug!("unrecognized action {action:#?}");
        return;
    };
        let state = match self.get(&action) {
            Some(ActionState::Inactive | ActionState::JustInactive) | None => {
                ActionState::JustActive
            }
            Some(ActionState::JustActive | ActionState::Active) => ActionState::Active,
        };
        self.insert(action, state);
    }

    pub fn key_up<A: TryInto<Action> + std::fmt::Debug + Copy>(&mut self, action: A) {
        let Ok(action) = action.try_into() else {
        log::debug!("unrecognized key {action:#?}");
        return;
    };
        let state = match self.get(&action) {
            Some(ActionState::Active | ActionState::JustActive) => ActionState::JustInactive,
            Some(ActionState::Inactive | ActionState::JustInactive) => ActionState::Inactive,
            None => ActionState::JustInactive,
        };
        self.insert(action, state);
    }
}
