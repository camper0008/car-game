#![warn(clippy::unwrap_used)]
#![allow(clippy::cast_possible_truncation)]

mod cli;
mod gear;
mod hand;
mod input;
mod lerp;
mod macros;

use cli::{Cli, Parser};
use gear::{Gear, Speed};
use hand::{clamp, Hand};
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

fn draw_gear_state(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    gear: &Gear,
) -> Result<(), String> {
    let state = gear.state();

    let initial_x = 128;
    let initial_y = 64;

    let (x, y) = match state {
        Speed::Neutral => (0, 0),
        Speed::Rocket => (1, 0),
        Speed::First => (0, 1),
        Speed::Second => (1, 1),
        Speed::Third => (0, 2),
        Speed::Fourth => (1, 2),
        Speed::Fifth => (0, 3),
    };

    canvas.copy(
        texture,
        rect!(initial_x + x * 32, initial_y + y * 16, 32, 16),
        rect!(position.0, position.1, 256, 128),
    )?;

    Ok(())
}

fn draw_keyboard_or_controller(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    is_keyboard: bool,
) -> Result<(), String> {
    let offset = i32::from(is_keyboard);

    canvas.copy(
        texture,
        rect!(192, 16 * offset, 32, 16),
        rect!(position.0, position.1, 256, 128),
    )?;

    Ok(())
}

fn draw_tachometer(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    angle: f64,
) -> Result<(), String> {
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

fn update_tachometer_angle(angle: &mut f64, accelerating: bool) {
    let min_angle = 307.0;
    let max_angle = 15.0;

    let acceleration_rate = *angle - min_angle;
    let acceleration_rate = -acceleration_rate;
    let acceleration_rate = acceleration_rate * 0.01 + 1.0;
    let acceleration_rate = if acceleration_rate > 5.0 {
        2.0
    } else {
        acceleration_rate
    };

    if accelerating {
        *angle -= 1.5 * acceleration_rate;
    } else {
        *angle += 0.5 * acceleration_rate;
    }
    if *angle > min_angle {
        *angle = min_angle;
    } else if *angle < max_angle {
        *angle = max_angle + 5.0;
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
                let Ok(key) = key.try_into() else {
                        println!("unrecognized key {key:#?}");
                        continue;
                    };
                let state = match input.get(&key) {
                    Some(ActionState::Inactive | ActionState::JustInactive) | None => {
                        ActionState::JustActive
                    }
                    Some(ActionState::JustActive | ActionState::Active) => ActionState::Active,
                };
                input.insert(key, state);
            }
            Event::ControllerButtonDown {
                timestamp: _,
                which: _,
                button,
            } => {
                let Ok(key) = button.try_into() else {
                        println!("unrecognized key {button:#?}");
                        continue;
                    };
                let state = match input.get(&key) {
                    Some(ActionState::Inactive | ActionState::JustInactive) | None => {
                        ActionState::JustActive
                    }
                    Some(ActionState::JustActive | ActionState::Active) => ActionState::Active,
                };
                input.insert(key, state);
            }

            Event::KeyUp {
                keycode: Some(key), ..
            } => {
                let Ok(key) = key.try_into() else {
                        println!("unrecognized key {key:#?}");
                        continue;
                    };
                let state = match input.get(&key) {
                    Some(ActionState::Active | ActionState::JustActive) => {
                        ActionState::JustInactive
                    }
                    Some(ActionState::Inactive | ActionState::JustInactive) => {
                        ActionState::Inactive
                    }
                    None => unreachable!(),
                };
                input.insert(key, state);
            }
            Event::ControllerButtonUp {
                timestamp: _,
                which: _,
                button,
            } => {
                let Ok(key) = button.try_into() else {
                        println!("unrecognized key {button:#?}");
                        continue;
                    };
                let state = match input.get(&key) {
                    Some(ActionState::Active | ActionState::JustActive) => {
                        ActionState::JustInactive
                    }
                    Some(ActionState::Inactive | ActionState::JustInactive) => {
                        ActionState::Inactive
                    }
                    None => unreachable!(),
                };
                input.insert(key, state);
            }

            Event::ControllerAxisMotion {
                timestamp: _,
                which: _,
                axis,
                value,
            } => match axis {
                Axis::RightX => input.update_right_joystick_from_raw_x(value),
                Axis::RightY => input.update_right_joystick_from_raw_y(value),
                _ => {}
            },
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

    let mut tachometer_angle: f64 = 360.0;

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

    let mut input = Input::new();

    let controller = match check_for_controllers(&controller_system) {
        Ok(controller) => Some(controller),
        Err(err) => {
            println!("error connecting controller: {err}");
            None
        }
    };

    let gearstick_position = (width - 128 * 4, padded_end(height, 160));

    'game_loop: loop {
        canvas.set_draw_color(Color::RGB(1, 25, 54));
        canvas.clear();

        draw_keyboard_or_controller(&mut canvas, &texture, (0, 0), controller.is_none())?;

        draw_tachometer(
            &mut canvas,
            &texture,
            (128, padded_end(height, 256)),
            tachometer_angle.to_radians(),
        )?;

        let smoothed_gear_offset = lerp::two_dimensional(gear.alpha, gear.offset, gear.target);

        draw_gearstick(
            &mut canvas,
            &texture,
            gearstick_position,
            smoothed_gear_offset,
        )?;

        let smoothed_hand_offset = lerp::two_dimensional(hand.alpha, hand.offset, hand.target);

        draw_hand(
            &mut canvas,
            &texture,
            gearstick_position,
            smoothed_hand_offset,
            input.action_active(&Action::Grab),
        )?;

        draw_gear_state(
            &mut canvas,
            &texture,
            (center(width, 256), padded_end(height, 128)),
            &gear,
        )?;

        canvas.present();

        poll_events(&sdl_context, &mut input)?;

        if input.get(&Action::Quit).is_some() {
            break 'game_loop Ok(());
        }

        hand.target = if controller.is_some() {
            Hand::gamepad_target(&input)
        } else {
            Hand::keyboard_target(&input)
        };

        if gear.held {
            let target = clamp(hand.target, hand.offset);
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
        } else if controller.is_some() {
            hand.reset(0.25, smoothed_hand_offset);
            gear.reset(0.25, smoothed_gear_offset);
        }

        update_tachometer_angle(
            &mut tachometer_angle,
            input.action_active(&Action::Accelerate),
        );

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

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
