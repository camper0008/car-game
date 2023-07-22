#![warn(clippy::unwrap_used)]

mod gear;
mod hand;
mod key_map;
mod lerp;
mod macros;

use gear::Gear;
use hand::{clamp, Hand};
use key_map::{change_key, key_changed, key_down, KeyCode, KeyMap};
use lerp::lerp2d;
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
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::key_map::KeyState;

fn prepare_window(sdl_context: &Sdl) -> Result<Window, String> {
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let mut window = video_subsystem
        .window("car-demo", 1920, 800)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    window.set_fullscreen(sdl2::video::FullscreenType::Off)?;
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
        gear::GearState::Neutral => (0, 0),
        gear::GearState::Rocket => (1, 0),
        gear::GearState::First => (0, 1),
        gear::GearState::Second => (1, 1),
        gear::GearState::Third => (0, 2),
        gear::GearState::Fourth => (1, 2),
        gear::GearState::Fifth => (0, 3),
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
    let offset = if is_keyboard { 1 } else { 0 };

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

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let controller_system = sdl_context.game_controller()?;
    let window = prepare_window(&sdl_context)?;
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

    let mut key_map: KeyMap = HashMap::new();

    let mut controller_right_x = 0;
    let mut controller_right_y = 0;

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

        hand.target = if controller.is_some() {
            Hand::gamepad_target(controller_right_x, controller_right_y)
        } else {
            Hand::keyboard_target(&key_map)
        };

        gear.target = if gear.held {
            clamp(hand.target, hand.offset)
        } else {
            gear.resting_target()
        };

        let smoothed_gear_offset = lerp2d(gear.alpha, gear.offset, gear.target);

        draw_gearstick(
            &mut canvas,
            &texture,
            gearstick_position,
            smoothed_gear_offset,
        )?;

        if gear.held {
            hand.target = clamp(hand.target, hand.offset)
        }

        let smoothed_hand_offset = lerp2d(hand.alpha, hand.offset, hand.target);

        draw_hand(
            &mut canvas,
            &texture,
            gearstick_position,
            smoothed_hand_offset,
            key_down(&key_map, &KeyCode::Space),
        )?;

        draw_gear_state(
            &mut canvas,
            &texture,
            (center(width, 256), padded_end(height, 128)),
            &gear,
        )?;

        canvas.present();

        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'game_loop Ok(()),
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    let Ok(key) = key.try_into() else {
                        println!("unrecognized key {key:#?}");
                        continue;
                    };
                    let state = match key_map.get(&key) {
                        Some(KeyState::Up | KeyState::WasDown) | None => KeyState::WasUp,
                        Some(KeyState::WasUp | KeyState::Down) => KeyState::Down,
                    };
                    key_map.insert(key, state);
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    let Ok(key) = key.try_into() else {
                        println!("unrecognized key {key:#?}");
                        continue;
                    };
                    let state = match key_map.get(&key) {
                        Some(KeyState::Down | KeyState::WasUp) => KeyState::WasDown,
                        Some(KeyState::Up | KeyState::WasDown) => KeyState::Up,
                        None => unreachable!(),
                    };
                    key_map.insert(key, state);
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
                    let state = match key_map.get(&key) {
                        Some(KeyState::Up | KeyState::WasDown) | None => KeyState::WasUp,
                        Some(KeyState::WasUp | KeyState::Down) => KeyState::Down,
                    };
                    key_map.insert(key, state);
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
                    let state = match key_map.get(&key) {
                        Some(KeyState::Down | KeyState::WasUp) => KeyState::WasDown,
                        Some(KeyState::Up | KeyState::WasDown) => KeyState::Up,
                        None => unreachable!(),
                    };
                    key_map.insert(key, state);
                }
                Event::ControllerAxisMotion {
                    timestamp: _,
                    which: _,
                    axis,
                    value,
                } => match axis {
                    Axis::RightX => controller_right_x = value,
                    Axis::RightY => controller_right_y = value,
                    _ => {}
                },
                _ => (),
            }
        }

        hand.tick(12.0 / 60.0);
        gear.tick(12.0 / 60.0);

        if Hand::has_changed(&key_map) {
            hand.reset(0.0, smoothed_hand_offset);
            gear.reset(0.0, smoothed_gear_offset);
        } else if controller.is_some() {
            hand.reset(0.25, smoothed_hand_offset);
            gear.reset(0.25, smoothed_gear_offset);
        }

        update_tachometer_angle(&mut tachometer_angle, key_down(&key_map, &KeyCode::Shift));

        if key_changed(&key_map, &KeyCode::Space) {
            let x_square = (smoothed_hand_offset.0 - smoothed_gear_offset.0).powi(2);
            let y_square = (smoothed_hand_offset.1 - smoothed_gear_offset.1).powi(2);
            let distance = (x_square + y_square).sqrt();

            if distance < 0.5 && key_down(&key_map, &KeyCode::Space) {
                gear.held = true;
                gear.reset(0.0, smoothed_gear_offset);
            } else {
                gear.held = false;
            }
        }
        Hand::update_keys(&mut key_map);
        change_key(&mut key_map, KeyCode::Space);

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
