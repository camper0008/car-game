use crate::utils;

pub const NORMALIZED_RPM: f64 = 208.78;
pub const REAR_GEAR_RATIO: f64 = 3.23;
pub const TIRE_DIAMETER: f64 = 26.5;

pub struct Gear {
    pub alpha: f64,
    pub held: bool,
    pub offset: (f64, f64),
    pub target: (f64, f64),
}

pub fn expected_kmh(rpm: f64, trans_gear_ratio: f64) -> f64 {
    (rpm * TIRE_DIAMETER) / (NORMALIZED_RPM * REAR_GEAR_RATIO * trans_gear_ratio)
}

pub fn expected_rpm(kmh: f64, trans_gear_ratio: f64) -> f64 {
    (NORMALIZED_RPM * kmh * REAR_GEAR_RATIO * trans_gear_ratio) / TIRE_DIAMETER
}

impl Gear {
    pub fn resting_target(&self) -> (f64, f64) {
        let (x, y) = self.offset;
        let x = if (-0.5..=0.5).contains(&y) {
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
        let (x, y) = utils::lerp_2d(self.alpha, self.offset, self.target);

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

#[derive(PartialEq)]
pub enum Speed {
    Neutral,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Rocket,
}

impl Speed {
    pub fn gear_ratio(&self) -> f64 {
        match self {
            Speed::Neutral => 3.55,
            Speed::First => 3.55,
            Speed::Second => 1.92,
            Speed::Third => 1.32,
            Speed::Fourth => 1.0,
            Speed::Fifth => 0.82,
            Speed::Rocket => 3.58,
        }
    }
}
