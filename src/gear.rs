use crate::lerp;

pub struct Gear {
    pub alpha: f64,
    pub held: bool,
    pub offset: (f64, f64),
    pub target: (f64, f64),
}

impl Gear {
    pub fn resting_target(&self) -> (f64, f64) {
        let (x, y) = self.offset;
        let x = if y <= 0.5 && y >= -0.5 {
            0.0
        } else if x >= 0.9 {
            1.0
        } else if x <= -0.9 {
            -1.0
        } else if !(-0.25..=0.25).contains(&x) {
            x
        } else {
            0.0
        };

        let y = if y >= 0.9 {
            1.0
        } else if y <= -0.9 {
            -1.0
        } else if y >= 0.5 {
            0.75
        } else if y <= -0.5 {
            -0.75
        } else {
            0.0
        };

        (x, y)
    }

    pub fn state(&self) -> Speed {
        let (x, y) = lerp::two_dimensional(self.alpha, self.offset, self.target);

        if (-0.9..=0.9).contains(&y) {
            return Speed::Neutral;
        }
        if (0.25..=0.9).contains(&x) {
            return Speed::Neutral;
        }
        if (-0.9..=-0.25).contains(&x) {
            return Speed::Neutral;
        }

        if x < -0.9 {
            if y <= -0.9 {
                Speed::First
            } else {
                Speed::Second
            }
        } else if x < 0.25 {
            if y <= -0.9 {
                Speed::Third
            } else {
                Speed::Fourth
            }
        } else if y <= -0.9 {
            Speed::Fifth
        } else {
            Speed::Rocket
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

pub enum Speed {
    Neutral,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Rocket,
}
