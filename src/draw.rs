use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use crate::gear::{Gear, Speed};
use crate::rect;

pub fn gearstick(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    offset: (f64, f64),
) -> Result<(), String> {
    canvas.copy(
        texture,
        rect!(128, 0, 64, 64),
        rect!(position.0, position.1, 160, 160),
    )?;

    let start_x = position.0 + 80;
    let start_y = position.1 + 80;
    let end_x = position.0 + 80 + (offset.0 * 128.0) as i16;
    let end_y = position.1 + 80 + (offset.1 * 128.0) as i16;

    if !(start_x == end_x && start_y == end_y) {
        canvas.filled_circle(start_x, start_y, 32, Color::RGB(178, 16, 48))?;
        canvas.thick_line(start_x, start_y, end_x, end_y, 64, Color::RGB(178, 16, 48))?;
    }

    canvas.copy(
        texture,
        rect!(64, 0, 64, 64),
        rect!(
            f64::from(position.0) + offset.0 * 128.0,
            f64::from(position.1) + offset.1 * 128.0,
            160,
            160
        ),
    )?;

    Ok(())
}

pub fn hand(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    offset: (f64, f64),
    grabbing: bool,
) -> Result<(), String> {
    let sprite_offset = if grabbing { 64 } else { 0 };

    canvas.copy(
        texture,
        rect!(sprite_offset, 64, 64, 64),
        rect!(
            f64::from(position.0) + offset.0 * 128.0,
            f64::from(position.1) + offset.1 * 128.0,
            128 + 32,
            128 + 32
        ),
    )?;

    Ok(())
}

pub fn clutch(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    is_clutched: bool,
) -> Result<(), String> {
    let texture_x = if is_clutched { 224 } else { 192 };

    canvas.copy(
        texture,
        rect!(texture_x, 32, 32, 32),
        rect!(position.0, position.1, 256, 256),
    )?;

    Ok(())
}

pub fn gear_state(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    gear: &Gear,
    is_clutched: bool,
) -> Result<(), String> {
    let state = if is_clutched {
        Speed::Neutral
    } else {
        gear.speed()
    };

    let initial_x = 128;
    let initial_y = 64;

    let y = match state {
        Speed::Neutral => 0,
        Speed::Rocket => 1,
        Speed::First => 2,
        Speed::Second => 3,
        Speed::Third => 4,
        Speed::Fourth => 5,
        Speed::Fifth => 6,
    };

    canvas.copy(
        texture,
        rect!(initial_x, initial_y + y * 6, 24, 5),
        rect!(position.0, position.1, 192, 40),
    )?;

    Ok(())
}

#[derive(Clone)]
enum Digit {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    None,
}

impl Digit {
    fn digit_to_texture(&self, negative: bool) -> (u32, u32) {
        let (x, y) = match self {
            Digit::Zero => (0, 0),
            Digit::One => (1, 0),
            Digit::Two => (2, 0),
            Digit::Three => (3, 0),
            Digit::Four => (0, 1),
            Digit::Five => (1, 1),
            Digit::Six => (2, 1),
            Digit::Seven => (3, 1),
            Digit::Eight => (0, 2),
            Digit::Nine => (1, 2),
            Digit::None => (2, 2),
        };

        let start_x = if negative { 176 } else { 160 };
        let start_y = 64;

        let x = (x * 4) + start_x;
        let y = y * 5 + start_y;

        (x, y)
    }
}

impl From<i64> for Digit {
    fn from(value: i64) -> Self {
        let values = [
            Self::Zero,
            Self::One,
            Self::Two,
            Self::Three,
            Self::Four,
            Self::Five,
            Self::Six,
            Self::Seven,
            Self::Eight,
            Self::Nine,
        ];
        if !(0..=9).contains(&value) {
            unreachable!("value {value} should be 0 >= {value} >= 9");
        }
        let variant = values[value as usize].clone();
        variant
    }
}

fn get_digit(value: i64, digit_place: i64) -> Digit {
    if value >= digit_place {
        let value = value % (digit_place * 10);
        let digit = (value - value % digit_place) / digit_place;
        let digit: Digit = digit.into();
        digit
    } else {
        Digit::None
    }
}

fn draw_digit(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    digit: &Digit,
    position: (i16, i16),
    negative: bool,
) -> Result<(), String> {
    let (texture_x, texture_y) = digit.digit_to_texture(negative);

    canvas.copy(
        texture,
        rect!(texture_x, texture_y, 3, 5),
        rect!(position.0, position.1, 24, 40),
    )?;

    Ok(())
}

pub fn kmh(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    kmh: f64,
) -> Result<(), String> {
    let kmh: i64 = kmh.round() as i64;
    let negative = kmh.is_negative();
    let kmh = kmh.abs();
    let first_digit: Digit = (kmh % 10).into();
    let second_digit = get_digit(kmh, 10);
    let third_digit = get_digit(kmh, 100);

    canvas.copy(
        texture,
        rect!(128, 106, 20, 5),
        rect!(position.0, position.1, 160, 40),
    )?;

    draw_digit(
        canvas,
        texture,
        &first_digit,
        (position.0 + 160 + 96 - 8, position.1),
        negative,
    )?;
    draw_digit(
        canvas,
        texture,
        &second_digit,
        (position.0 + 160 + 64 - 8, position.1),
        negative && (0..10).contains(&kmh),
    )?;
    draw_digit(
        canvas,
        texture,
        &third_digit,
        (position.0 + 160 + 32 - 8, position.1),
        negative && (10..100).contains(&kmh),
    )?;

    Ok(())
}

pub fn tachometer(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    rpm: f64,
) -> Result<(), String> {
    let min_rpm = 0.0;
    let max_rpm = 8000.0;
    let min_angle = -12.5;
    let max_angle = -347.5;

    let percentage = (rpm - min_rpm) / (max_rpm - min_rpm);

    let angle = (percentage * (max_angle - min_angle)) + min_angle;
    let angle = angle.to_radians();

    canvas.copy(
        texture,
        rect!(0, 0, 64, 64),
        rect!(position.0, position.1, 256, 256),
    )?;

    let center = (position.0 + 128, position.1 + 128);
    let offset = (angle.sin() * 116.0, angle.cos() * 116.0);
    let background_offset = (angle.sin() * 118.0, angle.cos() * 118.0);
    let target = (center.0 + offset.0 as i16, center.1 + offset.1 as i16);
    let background_target = (
        center.0 + background_offset.0 as i16,
        center.1 + background_offset.1 as i16,
    );
    canvas.thick_line(
        center.0,
        center.1,
        background_target.0,
        background_target.1,
        8,
        Color::RGB(127, 127, 127),
    )?;

    canvas.thick_line(
        center.0,
        center.1,
        target.0,
        target.1,
        4,
        Color::RGB(178, 16, 48),
    )?;

    Ok(())
}
