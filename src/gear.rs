use crate::lerp::lerp2d;

pub struct Gear {
    pub alpha: f64,
    pub held: bool,
    pub offset: (f64, f64),
    pub target: (f64, f64),
}

impl Gear {
    pub fn resting_target(&self) -> (f64, f64) {
        let (x, y) = self.offset;
        let x = if y < 0.5 && y > -0.5 {
            0.0
        } else if x > 0.9 {
            1.0
        } else if x < -0.9 {
            -1.0
        } else if !(-0.25..=0.25).contains(&x) {
            x
        } else {
            0.0
        };

        let y = if y > 0.9 {
            1.0
        } else if y < -0.9 {
            -1.0
        } else if y > 0.5 {
            0.75
        } else if y < -0.5 {
            -0.75
        } else {
            0.0
        };

        (x, y)
    }

    pub fn state(&self) -> GearState {
        let (x, y) = lerp2d(self.alpha, self.offset, self.target);

        if (-0.9..=0.9).contains(&y) {
            return GearState::Neutral;
        }
        if (0.25..=0.9).contains(&x) {
            return GearState::Neutral;
        }
        if (-0.9..=-0.25).contains(&x) {
            return GearState::Neutral;
        }

        if x < -0.9 {
            if y <= -0.9 {
                GearState::First
            } else {
                GearState::Second
            }
        } else if x < 0.25 {
            if y <= -0.9 {
                GearState::Third
            } else {
                GearState::Fourth
            }
        } else if y <= -0.9 {
            GearState::Fifth
        } else {
            GearState::Rocket
        }
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

pub enum GearState {
    Neutral,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Rocket,
}
