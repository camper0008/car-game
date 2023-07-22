#![warn(clippy::unwrap_used)]

mod gear;
mod hand;
mod key_map;
mod macros;

use gear::{gear_resting_target, gear_state};
use hand::{change_hand, clamp_hand, hand_changed, hand_target};
use key_map::{change_key, key_changed, key_down, KeyCode, KeyMap};
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
        .window("car-demo", 1000, 800)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    window.set_fullscreen(sdl2::video::FullscreenType::Desktop)?;
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
    target: (f64, f64),
    alpha: f64,
) -> Result<(f64, f64), String> {
    canvas.copy(
        texture,
        rect!(128, 0, 64, 64),
        rect!(position.0, position.1, 160, 160),
    )?;

    let x = lerp(alpha, offset.0, target.0);
    let y = lerp(alpha, offset.1, target.1);

    let start_x = position.0 + 80;
    let start_y = position.1 + 80;
    let end_x = position.0 + 80 + (x * 128.0) as i16;
    let end_y = position.1 + 80 + (y * 128.0) as i16;

    if !(start_x == end_x && start_y == end_y) {
        canvas.filled_circle(start_x, start_y, 32, Color::RGB(178, 16, 48))?;
        canvas.thick_line(start_x, start_y, end_x, end_y, 64, Color::RGB(178, 16, 48))?;
    }

    canvas.copy(
        texture,
        rect!(64, 0, 64, 64),
        rect!(
            f64::from(position.0) + x * 128.0,
            f64::from(position.1) + y * 128.0,
            160,
            160
        ),
    )?;

    Ok((x, y))
}

fn lerp(alpha: f64, position: f64, target: f64) -> f64 {
    position + alpha * (target - position)
}

fn draw_hand(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    offset: (f64, f64),
    target: (f64, f64),
    alpha: f64,
    grabbing: bool,
) -> Result<(f64, f64), String> {
    let sprite_offset = if grabbing { 64 } else { 0 };

    let x = lerp(alpha, offset.0, target.0);
    let y = lerp(alpha, offset.1, target.1);

    canvas.copy(
        texture,
        rect!(sprite_offset, 64, 64, 64),
        rect!(
            f64::from(position.0) + x * 128.0,
            f64::from(position.1) + y * 128.0,
            128 + 32,
            128 + 32
        ),
    )?;

    Ok((x, y))
}

fn draw_gear_state(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    position: (i16, i16),
    gear: (f64, f64),
) -> Result<(), String> {
    let state = gear_state(gear);

    let initial_x = 128;
    let initial_y = 64;

    let (x, y) = match state {
        gear::Gear::Neutral => (0, 0),
        gear::Gear::Rocket => (1, 0),
        gear::Gear::First => (0, 1),
        gear::Gear::Second => (1, 1),
        gear::Gear::Third => (0, 2),
        gear::Gear::Fourth => (1, 2),
        gear::Gear::Fifth => (0, 3),
    };

    canvas.copy(
        texture,
        rect!(initial_x + x * 32, initial_y + y * 16, 32, 16),
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

    let mut hand_alpha = 0.0;
    let mut hand_offset = (0.0, 0.0);

    let mut gear_alpha = 0.0;
    let mut gear_offset = (0.0, 0.0);
    let mut gear_held = false;

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

    'game_loop: loop {
        canvas.set_draw_color(Color::RGB(1, 25, 54));
        canvas.clear();

        draw_tachometer(
            &mut canvas,
            &texture,
            (128, padded_end(height, 256)),
            tachometer_angle.to_radians(),
        )?;

        let hand_target = if controller.is_some() {
            let x = (f64::from(controller_right_x) / f64::from(i16::MAX)) * 1.5;
            let y = (f64::from(controller_right_y) / f64::from(i16::MAX)) * 1.5;
            let x = if x > 1.0 {
                1.0
            } else if x < -1.0 {
                -1.0
            } else {
                x
            };
            let y = if y > 1.0 {
                1.0
            } else if y < -1.0 {
                -1.0
            } else {
                y
            };

            (x, y)
        } else {
            hand_target(&key_map)
        };

        let gear_target = if gear_held {
            clamp_hand(hand_target, hand_offset)
        } else {
            gear_resting_target(gear_offset)
        };

        let new_gear_offset = draw_gearstick(
            &mut canvas,
            &texture,
            (width - 128 * 4, padded_end(height, 160)),
            gear_offset,
            gear_target,
            gear_alpha,
        )?;

        draw_gear_state(
            &mut canvas,
            &texture,
            (center(width, 256), padded_end(height, 128)),
            new_gear_offset,
        )?;

        let hand_target = if gear_held {
            clamp_hand(hand_target, hand_offset)
        } else {
            hand_target
        };

        let new_hand_offset = draw_hand(
            &mut canvas,
            &texture,
            (width - 128 * 4, padded_end(height, 160)),
            hand_offset,
            hand_target,
            hand_alpha,
            key_down(&key_map, &KeyCode::Space),
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
                _ => (),
            }
        }

        update_tachometer_angle(&mut tachometer_angle, key_down(&key_map, &KeyCode::Shift));

        hand_alpha += 12.0 / 60.0;
        if hand_alpha > 1.0 {
            hand_alpha = 1.0;
        }

        gear_alpha += 12.0 / 60.0;
        if gear_alpha > 1.0 {
            gear_alpha = 1.0;
        }

        if hand_changed(&key_map) {
            hand_alpha = 0.0;
            hand_offset = new_hand_offset;

            gear_alpha = 0.0;
            gear_offset = new_gear_offset;
        }

        if controller.is_some() {
            hand_alpha = 0.25;
            hand_offset = new_hand_offset;

            gear_alpha = 0.25;
            gear_offset = new_gear_offset;
        }

        if key_changed(&key_map, &KeyCode::Space) {
            let distance = ((new_hand_offset.0 - new_gear_offset.0).powi(2)
                + (new_hand_offset.1 - new_gear_offset.1).powi(2))
            .sqrt();

            if distance < 0.5 && key_down(&key_map, &KeyCode::Space) {
                gear_held = true;
            } else {
                gear_held = false;
            }
        }

        change_hand(&mut key_map);
        change_key(&mut key_map, KeyCode::Space);

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
