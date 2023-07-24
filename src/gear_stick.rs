use crate::utils;

pub const NORMALIZED_RPM: f64 = 208.78;
pub const REAR_GEAR_RATIO: f64 = 3.23;
pub const TIRE_DIAMETER: f64 = 26.5;

pub struct GearStick {
    pub smooth_factor: f64,
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

impl GearStick {
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

    pub fn gear(&self, is_clutched: bool) -> Gear {
        let (x, y) = utils::lerp_2d(self.smooth_factor, self.offset, self.target);

        if is_clutched {
            return Gear::Neutral;
        }
        if (-0.9..=0.9).contains(&y) {
            return Gear::Neutral;
        }
        if (0.25..=0.9).contains(&x) {
            return Gear::Neutral;
        }
        if (-0.9..=-0.25).contains(&x) {
            return Gear::Neutral;
        }

        if x < -0.9 {
            if y <= -0.9 {
                Gear::First
            } else {
                Gear::Second
            }
        } else if x < 0.25 {
            if y <= -0.9 {
                Gear::Third
            } else {
                Gear::Fourth
            }
        } else if y <= -0.9 {
            Gear::Fifth
        } else {
            Gear::Rocket
        }
    }

    pub fn set_origin(&mut self, offset: (f64, f64)) {
        self.offset = offset;
    }
}

#[derive(PartialEq)]
pub enum Gear {
    Neutral,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Rocket,
}

impl Gear {
    pub fn gear_ratio(&self) -> f64 {
        match self {
            Gear::First | Gear::Neutral => 3.55,
            Gear::Second => 1.92,
            Gear::Third => 1.32,
            Gear::Fourth => 1.0,
            Gear::Fifth => 0.82,
            Gear::Rocket => 3.58,
        }
    }
}
