#![warn(clippy::unwrap_used)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::too_many_lines)]

mod cli;
mod draw;
mod gear_stick;
mod hand;
mod input;
mod utils;

use cli::{Cli, Parser};
use gear_stick::{expected_kmh, expected_rpm, Gear, GearStick};
use hand::{clamp_clutch_down, clamp_clutch_up, Hand};
use input::{Action, Input};
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
    neutral_gear: bool,
    speeder_alpha: f64,
) -> f64 {
    let min_rpm = 700.0;
    let max_rpm = 8000.0;

    let rpm_to_accel = |rpm: f64| -rpm.powf(1.3) + 1.8 * rpm + 1.0;
    let rpm_to_deaccel = |rpm: f64| rpm.powf(1.1) + 1.0;

    let acceleration_rate = rpm_to_accel(rpm / 1000.0);
    let deacceleration_rate = if neutral_gear {
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
        min_rpm
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
            } => input.key_down(Keycode::Escape),
            Event::KeyDown {
                keycode: Some(key), ..
            } => input.key_down(key),
            Event::ControllerButtonDown {
                timestamp: _,
                which: _,
                button,
            } => input.key_down(button),
            Event::MouseButtonDown {
                timestamp: _,
                window_id: _,
                which: _,
                mouse_btn,
                clicks: _,
                x: _,
                y: _,
            } => input.key_down(mouse_btn),
            Event::KeyUp {
                keycode: Some(key), ..
            } => input.key_up(key),
            Event::ControllerButtonUp {
                timestamp: _,
                which: _,
                button,
            } => input.key_up(button),
            Event::MouseButtonUp {
                timestamp: _,
                window_id: _,
                which: _,
                mouse_btn,
                clicks: _,
                x: _,
                y: _,
            } => input.key_up(mouse_btn),
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
                    input.brake_alpha = f64::from(value) / f64::from(i16::MAX);
                }
                Axis::TriggerRight => {
                    if value < 100 {
                        input.key_up(Keycode::Up);
                    } else {
                        input.key_down(Keycode::Up);
                    }
                    input.speeder_alpha = f64::from(value) / f64::from(i16::MAX);
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
            } => input.update_hand_relatively(xrel, yrel),
            Event::ControllerDeviceAdded {
                timestamp: _,
                which,
            } => match controllers.open(which).map_err(|e| e.to_string()) {
                Ok(controller) => input.active_controller = Some(controller),
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

impl Default for ClutchCooldown {
    fn default() -> Self {
        Self {
            start_rpm: 0.0,
            timer: 0.0,
            active: false,
        }
    }
}

fn window_size(window: &Window) -> Result<(i16, i16), String> {
    let (width, height) = window.size();
    let size = (
        i16::try_from(width).map_err(|e| e.to_string())?,
        i16::try_from(height).map_err(|e| e.to_string())?,
    );

    Ok(size)
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
    let (width, height) = window_size(&window)?;
    let mut canvas = prepare_canvas(window)?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture(Path::new("assets/tile.png"))?;

    let mut rpm: f64 = 0.0;
    let mut kmh: f64 = 0.0;
    let mut previous_gear = Gear::Neutral;
    let mut clutch_cooldown = ClutchCooldown::default();
    let mut hand = Hand::default();
    let mut gear_stick = GearStick::default();
    let mut input = Input::with_sensitivity(cli.mouse_sensitivity);

    match check_for_controllers(&mut input, &controller_system) {
        Ok(_) => log::info!("controller connected"),
        Err(err) => {
            log::debug!("error connecting controller: {err}");
        }
    };

    'game_loop: loop {
        canvas.set_draw_color(Color::RGB(1, 25, 54));
        canvas.clear();
        sdl_context.mouse().set_relative_mouse_mode(true);

        let hand_offset = hand.next_offset();
        hand.set_origin(hand_offset);

        let gear_stick_offset = gear_stick.next_offset();
        gear_stick.set_origin(gear_stick_offset);

        let gear = gear_stick.gear(input.action_active(&Action::Clutch));

        draw::all(
            &mut canvas,
            &texture,
            (width, height),
            &draw::Peripherals {
                rpm,
                kmh,
                gear: &gear,
            },
            gear_stick_offset,
            &draw::Hand {
                offset: hand_offset,
                grabbing: input.action_active(&Action::Grab),
            },
            &draw::Pedals {
                speeder_down: input.action_active(&Action::Accelerate),
                clutch_down: input.action_active(&Action::Clutch),
                brake_down: input.action_active(&Action::Brake),
            },
        )?;

        canvas.present();

        poll_events(&sdl_context, &mut input, &controller_system)?;

        if input.action_active(&Action::Quit) {
            break 'game_loop Ok(());
        }

        hand.target = Hand::target(&input);
        if gear_stick.held {
            let target = if input.action_active(&Action::Clutch) {
                clamp_clutch_down(hand.target, hand.offset)
            } else {
                clamp_clutch_up(hand.target, hand.offset, &gear)
            };
            hand.target = target;
            gear_stick.target = target;
        } else {
            gear_stick.target = gear_stick.resting_target();
        };

        if input.action_active(&Action::Brake) {
            let speed_diff = (50.0 / 60.0) * input.brake_alpha;
            kmh -= speed_diff;

            if !(gear == Gear::Neutral) {
                let rpm_before = expected_rpm(kmh, gear.gear_ratio());
                let rpm_after = expected_rpm(kmh - speed_diff, gear.gear_ratio());
                rpm -= rpm_before - rpm_after;
            }
        }

        if gear == Gear::Neutral {
            clutch_cooldown.active = false;
            clutch_cooldown.timer = 0.0;

            let speeder_down = input.action_active(&Action::Accelerate);
            let neutral_gear = gear == Gear::Neutral;
            let speeder_alpha = input.speeder_alpha;
            rpm = flywheel_rpm(rpm, &mut input, speeder_down, neutral_gear, speeder_alpha);
            kmh -= 1.0 / 60.0;
            if kmh < expected_kmh(700.0, Gear::Neutral.gear_ratio()) {
                kmh = expected_kmh(700.0, Gear::Neutral.gear_ratio());
            }
        } else if previous_gear == Gear::Neutral && !clutch_cooldown.active {
            clutch_cooldown.active = true;
            clutch_cooldown.start_rpm = rpm;
        } else if clutch_cooldown.timer < 1.0 && clutch_cooldown.active {
            let target = expected_rpm(kmh, gear.gear_ratio());
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
            let neutral_gear = gear == Gear::Neutral;
            let speeder_alpha = input.speeder_alpha;
            rpm = flywheel_rpm(rpm, &mut input, speeder_down, neutral_gear, speeder_alpha);
            kmh = expected_kmh(rpm, gear.gear_ratio());
        }

        if input.action_changed(&Action::Grab) {
            let x_square = (hand_offset.0 - gear_stick_offset.0).powi(2);
            let y_square = (hand_offset.1 - gear_stick_offset.1).powi(2);
            let distance = (x_square + y_square).sqrt();

            gear_stick.held = input.action_active(&Action::Grab) && distance < 0.5;
        }

        input.action_tick(Action::Grab);
        input.action_tick(Action::Clutch);

        previous_gear = gear;

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
