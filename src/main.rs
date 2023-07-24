#![warn(clippy::unwrap_used)]
#![allow(clippy::cast_possible_truncation)]

mod cli;
mod gear;
mod hand;
mod input;
mod macros;
mod utils;

use cli::{Cli, Parser};
use gear::{expected_kmh, Gear, Speed};
use hand::{clamp_clutch_down, clamp_clutch_up, Hand};
use input::{Action, ActionState, Input};
use sdl2::controller::{Axis, GameController};
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::video::Window;
use sdl2::{GameControllerSubsystem, Sdl};
use std::path::Path;
use std::time::Duration;

fn prepare_window(sdl_context: &Sdl, fullscreen: bool) -> Result<Window, String> {
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let mut window = video_subsystem
        .window("car-demo", 1920, 800)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    if fullscreen {
        window.set_fullscreen(sdl2::video::FullscreenType::Desktop)?;
    }
    Ok(window)
}

fn prepare_canvas(window: Window) -> Result<WindowCanvas, String> {
    window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())
}

fn draw_gearstick(
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

fn draw_hand(
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

fn draw_clutch(
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

fn draw_gear_state(
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

fn draw_kmh(
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

fn draw_tachometer(
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

fn update_flywheel_rpm(rpm: &mut f64, accelerating: bool) {
    let min_rpm = 700.0;
    let max_rpm = 8000.0;

    let rpm_to_accel = |rpm: f64| -rpm.powf(1.3) + 1.8 * rpm + 1.0;
    let rpm_to_deaccel = |rpm: f64| rpm.powf(1.5) + 1.0;

    let acceleration_rate = rpm_to_accel(*rpm / 1000.0);
    let deacceleration_rate = rpm_to_deaccel(*rpm / 1000.0);

    if accelerating {
        *rpm += 1500.0 / 60.0 * acceleration_rate;
    } else {
        *rpm -= 250.0 / 60.0 * deacceleration_rate;
    }
    if min_rpm > *rpm {
        *rpm = min_rpm + 50.0;
    } else if *rpm > max_rpm {
        *rpm = max_rpm - 100.0;
    }
}

fn padded_end(max: i16, length: i16) -> i16 {
    max - 256 - length / 2
}

fn center(max: i16, length: i16) -> i16 {
    (max / 2) - length / 2
}

fn check_for_controllers(system: &GameControllerSubsystem) -> Result<GameController, String> {
    if system.num_joysticks()? == 0 {
        return Err("no controllers connected".to_string());
    }

    system.open(0).map_err(|e| e.to_string())
}

fn poll_events(sdl_context: &Sdl, input: &mut Input) -> Result<(), String> {
    for event in sdl_context.event_pump()?.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                input.insert(Action::Quit, ActionState::Active);
            }
            Event::KeyDown {
                keycode: Some(key), ..
            } => {
                input.key_down(key);
            }
            Event::ControllerButtonDown {
                timestamp: _,
                which: _,
                button,
            } => {
                input.key_down(button);
            }
            Event::MouseButtonDown {
                timestamp: _,
                window_id: _,
                which: _,
                mouse_btn,
                clicks: _,
                x: _,
                y: _,
            } => {
                input.key_down(mouse_btn);
            }
            Event::KeyUp {
                keycode: Some(key), ..
            } => {
                input.key_up(key);
            }
            Event::ControllerButtonUp {
                timestamp: _,
                which: _,
                button,
            } => {
                input.key_up(button);
            }
            Event::MouseButtonUp {
                timestamp: _,
                window_id: _,
                which: _,
                mouse_btn,
                clicks: _,
                x: _,
                y: _,
            } => {
                input.key_up(mouse_btn);
            }

            Event::ControllerAxisMotion {
                timestamp: _,
                which: _,
                axis,
                value,
            } => match axis {
                Axis::RightX => input.update_hand_from_raw_x(value),
                Axis::RightY => input.update_hand_from_raw_y(value),
                _ => {}
            },
            Event::MouseMotion {
                timestamp: _,
                window_id: _,
                which: _,
                mousestate: _,
                x: _,
                y: _,
                xrel,
                yrel,
            } => {
                input.update_hand_relatively(xrel, yrel);
            }
            _ => (),
        }
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let sdl_context = sdl2::init()?;
    let controller_system = sdl_context.game_controller()?;
    let window = prepare_window(&sdl_context, !cli.windowed)?;
    let (width, height) = window.size();
    let (width, height) = (
        i16::try_from(width).expect("invalid width"),
        i16::try_from(height).expect("invalid height"),
    );
    let mut canvas = prepare_canvas(window)?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture(Path::new("assets/tile.png"))?;

    let mut flywheel_rpm: f64 = 0.0;
    let mut kmh: f64 = 5.0;
    let mut previous_speed = Speed::Neutral;

    let mut hand = Hand {
        alpha: 0.0,
        offset: (0.0, 0.0),
        target: (0.0, 0.0),
    };

    let mut gear = Gear {
        alpha: 0.0,
        held: false,
        offset: (0.0, 0.0),
        target: (0.0, 0.0),
    };

    let mut input = Input::with_sensitivity(cli.mouse_sensitivity);

    if let Err(err) = check_for_controllers(&controller_system) {
        println!("error connecting controller: {err}");
    };

    let gearstick_position = (width - 128 * 4, padded_end(height, 160));

    'game_loop: loop {
        canvas.set_draw_color(Color::RGB(1, 25, 54));
        canvas.clear();
        sdl_context.mouse().set_relative_mouse_mode(true);

        draw_tachometer(
            &mut canvas,
            &texture,
            (128, padded_end(height, 256)),
            flywheel_rpm,
        )?;

        let smoothed_gear_offset = utils::lerp_2d(gear.alpha, gear.offset, gear.target);

        draw_gearstick(
            &mut canvas,
            &texture,
            gearstick_position,
            smoothed_gear_offset,
        )?;

        let smoothed_hand_offset = utils::lerp_2d(hand.alpha, hand.offset, hand.target);

        draw_hand(
            &mut canvas,
            &texture,
            gearstick_position,
            smoothed_hand_offset,
            input.action_active(&Action::Grab),
        )?;

        draw_clutch(
            &mut canvas,
            &texture,
            (center(width, 256), padded_end(height, 128)),
            input.action_active(&Action::Clutch),
        )?;

        draw_gear_state(
            &mut canvas,
            &texture,
            (center(width, 192), padded_end(height, 128) - 64),
            &gear,
            input.action_active(&Action::Clutch),
        )?;
        draw_kmh(
            &mut canvas,
            &texture,
            (center(width, 160 + 96), padded_end(height, 128) - 128),
            kmh,
        )?;

        canvas.present();

        poll_events(&sdl_context, &mut input)?;

        if input.get(&Action::Quit).is_some() {
            break 'game_loop Ok(());
        }

        hand.target = Hand::target(&input);

        if gear.held {
            let target = if input.action_active(&Action::Clutch) {
                clamp_clutch_down(hand.target, hand.offset)
            } else {
                clamp_clutch_up(hand.target, hand.offset, &gear.speed())
            };
            hand.target = target;
            gear.target = target;
        } else {
            gear.target = gear.resting_target();
        };

        hand.tick(12.0 / 60.0);
        gear.tick(12.0 / 60.0);

        if Hand::has_changed(&input) {
            hand.reset(0.0, smoothed_hand_offset);
            gear.reset(0.0, smoothed_gear_offset);
        } else {
            hand.reset(0.25, smoothed_hand_offset);
            gear.reset(0.25, smoothed_gear_offset);
        }

        update_flywheel_rpm(&mut flywheel_rpm, input.action_active(&Action::Accelerate));
        if gear.speed() == Speed::Neutral {
            kmh -= 1.0 / 60.0;
        } else {
            kmh = expected_kmh(flywheel_rpm, gear.speed().gear_ratio());
        }

        if input.action_changed(&Action::Grab) {
            let x_square = (smoothed_hand_offset.0 - smoothed_gear_offset.0).powi(2);
            let y_square = (smoothed_hand_offset.1 - smoothed_gear_offset.1).powi(2);
            let distance = (x_square + y_square).sqrt();

            if distance < 0.5 && input.action_active(&Action::Grab) {
                gear.held = true;
                gear.reset(0.0, smoothed_gear_offset);
            } else {
                gear.held = false;
            }
        }
        Hand::action_tick(&mut input);

        previous_speed = gear.speed();

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
