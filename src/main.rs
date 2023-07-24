#![warn(clippy::unwrap_used)]
#![allow(clippy::cast_possible_truncation)]

mod cli;
mod draw;
mod gear;
mod hand;
mod input;
mod macros;
mod utils;

use cli::{Cli, Parser};
use gear::{expected_kmh, expected_rpm, Gear, Speed};
use hand::{clamp_clutch_down, clamp_clutch_up, Hand};
use input::{Action, ActionState, Input};
use sdl2::controller::{Axis, GameController};
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use sdl2::{GameControllerSubsystem, Sdl};
use std::path::Path;
use std::time::Duration;
use utils::lerp_1d;

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
    let mut clutching_in_start_rpm = 0.0;
    let mut clutching_in_timer = 0.0;
    let mut clutching_in = false;

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

        draw::tachometer(
            &mut canvas,
            &texture,
            (128, padded_end(height, 256)),
            flywheel_rpm,
        )?;

        let smoothed_gear_offset = utils::lerp_2d(gear.alpha, gear.offset, gear.target);

        draw::gearstick(
            &mut canvas,
            &texture,
            gearstick_position,
            smoothed_gear_offset,
        )?;

        let smoothed_hand_offset = utils::lerp_2d(hand.alpha, hand.offset, hand.target);

        draw::hand(
            &mut canvas,
            &texture,
            gearstick_position,
            smoothed_hand_offset,
            input.action_active(&Action::Grab),
        )?;

        draw::clutch(
            &mut canvas,
            &texture,
            (center(width, 256), padded_end(height, 128)),
            input.action_active(&Action::Clutch),
        )?;

        draw::gear_state(
            &mut canvas,
            &texture,
            (center(width, 192), padded_end(height, 128) - 64),
            &gear,
            input.action_active(&Action::Clutch),
        )?;
        draw::kmh(
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

        if gear.speed() == Speed::Neutral {
            clutching_in = false;
            clutching_in_timer = 0.0;
            update_flywheel_rpm(&mut flywheel_rpm, input.action_active(&Action::Accelerate));
            kmh -= 1.0 / 60.0;
            if kmh < expected_kmh(700.0, Speed::Neutral.gear_ratio()) {
                kmh = expected_kmh(700.0, Speed::Neutral.gear_ratio());
            }
        } else if previous_speed == Speed::Neutral && !clutching_in {
            clutching_in = true;
            clutching_in_start_rpm = flywheel_rpm;
        } else if clutching_in_timer < 1.0 && clutching_in {
            flywheel_rpm = lerp_1d(
                clutching_in_timer,
                clutching_in_start_rpm,
                expected_rpm(kmh, gear.speed().gear_ratio()),
            );
            clutching_in_timer += 4.0 / 60.0;
        } else if clutching_in && clutching_in_timer >= 1.0 {
            clutching_in = false;
            clutching_in_timer = 0.0;
        } else {
            update_flywheel_rpm(&mut flywheel_rpm, input.action_active(&Action::Accelerate));
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
