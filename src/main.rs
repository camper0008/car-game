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
use draw::PedalState;
use gear::{expected_kmh, expected_rpm, Gear, Speed};
use hand::{clamp_clutch_down, clamp_clutch_up, Hand};
use input::{Action, ActionState, Input};
use sdl2::controller::Axis;
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

fn flywheel_rpm(
    rpm: f64,
    input: &mut Input,
    speeder_down: bool,
    clutch_down: bool,
    speeder_alpha: f64,
) -> f64 {
    let min_rpm = 700.0;
    let max_rpm = 8000.0;

    let rpm_to_accel = |rpm: f64| -rpm.powf(1.3) + 1.8 * rpm + 1.0;
    let rpm_to_deaccel = |rpm: f64| rpm.powf(1.3) + 1.0;

    let acceleration_rate = rpm_to_accel(rpm / 1000.0);
    let deacceleration_rate = if clutch_down {
        rpm_to_deaccel(rpm / 1000.0)
    } else {
        1.0
    };

    let rpm = if speeder_down {
        rpm + (1500.0 / 60.0) * acceleration_rate * speeder_alpha
    } else {
        rpm - 500.0 / 60.0 * deacceleration_rate
    };

    if min_rpm > rpm {
        min_rpm + 50.0
    } else if rpm > max_rpm {
        input.shake_controller();
        max_rpm - 100.0
    } else {
        rpm
    }
}

fn check_for_controllers(
    input: &mut Input,
    system: &GameControllerSubsystem,
) -> Result<(), String> {
    if system.num_joysticks()? == 0 {
        return Err("no controllers connected".to_string());
    }

    let controller = system.open(0).map_err(|e| e.to_string())?;
    input.active_controller = Some(controller);
    Ok(())
}

fn poll_events(
    sdl_context: &Sdl,
    input: &mut Input,
    controllers: &GameControllerSubsystem,
) -> Result<(), String> {
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
                Axis::TriggerLeft => {
                    if value < 100 {
                        input.key_up(Keycode::Down);
                    } else {
                        input.key_down(Keycode::Down);
                    }
                    input.brake_alpha = value as f64 / i16::MAX as f64;
                }
                Axis::TriggerRight => {
                    if value < 100 {
                        input.key_up(Keycode::Up);
                    } else {
                        input.key_down(Keycode::Up);
                    }
                    input.speeder_alpha = value as f64 / i16::MAX as f64;
                }
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
            Event::ControllerDeviceAdded {
                timestamp: _,
                which,
            } => match controllers.open(which).map_err(|e| e.to_string()) {
                Ok(controller) => {
                    input.active_controller = Some(controller);
                }
                Err(err) => log::error!("unable to connect controller: {err}"),
            },

            Event::ControllerDeviceRemoved {
                timestamp: _,
                which: _,
            } => input.active_controller = None,
            e => log::debug!("unrecognized event {e:?}"),
        }
    }

    Ok(())
}

struct ClutchCooldown {
    active: bool,
    start_rpm: f64,
    timer: f64,
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    simple_logger::SimpleLogger::new()
        .with_level(cli.log_level)
        .init()
        .map_err(|err| err.to_string())?;

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

    let mut rpm: f64 = 0.0;
    let mut kmh: f64 = 5.0;
    let mut previous_speed = Speed::Neutral;
    let mut clutch_cooldown = ClutchCooldown {
        start_rpm: 0.0,
        timer: 0.0,
        active: false,
    };

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

    match check_for_controllers(&mut input, &controller_system) {
        Ok(_) => log::info!("controller connected"),
        Err(err) => {
            log::warn!("error connecting controller: {err}")
        }
    };

    'game_loop: loop {
        canvas.set_draw_color(Color::RGB(1, 25, 54));
        canvas.clear();
        sdl_context.mouse().set_relative_mouse_mode(true);

        let smoothed_hand_offset = utils::lerp_2d(hand.alpha, hand.offset, hand.target);
        let smoothed_gear_offset = utils::lerp_2d(gear.alpha, gear.offset, gear.target);

        draw::all(
            &mut canvas,
            &texture,
            (width, height),
            rpm,
            kmh,
            smoothed_gear_offset,
            draw::HandState {
                offset: smoothed_hand_offset,
                grabbing: input.action_active(&Action::Grab),
            },
            PedalState {
                speeder_down: input.action_active(&Action::Accelerate),
                clutch_down: input.action_active(&Action::Clutch),
                brake_down: input.action_active(&Action::Brake),
            },
            gear.speed(),
        )?;
        canvas.present();

        poll_events(&sdl_context, &mut input, &controller_system)?;

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

        hand.reset(smoothed_hand_offset);
        gear.reset(smoothed_gear_offset);

        if input.action_active(&Action::Brake) {
            let speed_diff = (50.0 / 60.0) * input.brake_alpha;
            kmh -= speed_diff;

            if !(input.action_active(&Action::Clutch) || gear.speed() == Speed::Neutral) {
                let rpm_before = expected_rpm(kmh, gear.speed().gear_ratio());
                let rpm_after = expected_rpm(kmh - speed_diff, gear.speed().gear_ratio());
                rpm -= rpm_before - rpm_after;
            }
        }

        if gear.speed() == Speed::Neutral || input.action_active(&Action::Clutch) {
            clutch_cooldown.active = false;
            clutch_cooldown.timer = 0.0;

            let speeder_down = input.action_active(&Action::Accelerate);
            let clutch_down =
                input.action_active(&Action::Clutch) || gear.speed() == Speed::Neutral;
            let speeder_alpha = input.speeder_alpha;
            rpm = flywheel_rpm(rpm, &mut input, speeder_down, clutch_down, speeder_alpha);
            kmh -= 1.0 / 60.0;
            if kmh < expected_kmh(700.0, Speed::Neutral.gear_ratio()) {
                kmh = expected_kmh(700.0, Speed::Neutral.gear_ratio());
            }
        } else if previous_speed == Speed::Neutral && !clutch_cooldown.active {
            clutch_cooldown.active = true;
            clutch_cooldown.start_rpm = rpm;
        } else if clutch_cooldown.timer < 1.0 && clutch_cooldown.active {
            let target = expected_rpm(kmh, gear.speed().gear_ratio());
            rpm = lerp_1d(clutch_cooldown.timer, clutch_cooldown.start_rpm, target);
            clutch_cooldown.timer += 4.0 / 60.0;

            if (rpm - target).abs() > 500.0 {
                input.shake_controller();
            }
        } else if clutch_cooldown.active && clutch_cooldown.timer >= 1.0 {
            clutch_cooldown.active = false;
            clutch_cooldown.timer = 0.0;
        } else {
            let speeder_down = input.action_active(&Action::Accelerate);
            let clutch_down =
                input.action_active(&Action::Clutch) || gear.speed() == Speed::Neutral;
            let speeder_alpha = input.speeder_alpha;
            rpm = flywheel_rpm(rpm, &mut input, speeder_down, clutch_down, speeder_alpha);
            kmh = expected_kmh(rpm, gear.speed().gear_ratio());
        }

        if input.action_changed(&Action::Grab) {
            let x_square = (smoothed_hand_offset.0 - smoothed_gear_offset.0).powi(2);
            let y_square = (smoothed_hand_offset.1 - smoothed_gear_offset.1).powi(2);
            let distance = (x_square + y_square).sqrt();

            if distance < 0.5 && input.action_active(&Action::Grab) {
                gear.held = true;
                gear.reset(smoothed_gear_offset);
            } else {
                gear.held = false;
            }
        }

        input.action_tick(Action::Grab);
        input.action_tick(Action::Clutch);

        previous_speed = if input.action_active(&Action::Clutch) {
            Speed::Neutral
        } else {
            gear.speed()
        };

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
