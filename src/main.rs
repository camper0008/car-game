#![warn(clippy::unwrap_used)]

use sdl2::event::Event;
use sdl2::gfx::rotozoom::RotozoomSurface;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::path::Path;
use std::time::Duration;

fn prepare_window(sdl_context: &Sdl) -> Result<Window, String> {
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("car-demo", 640, 640)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    Ok(window)
}

fn prepare_canvas(window: Window) -> Result<WindowCanvas, String> {
    window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())
}

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn draw_tachometer_background<'a>(surface: &'a Surface<'a>) -> Result<Surface<'_>, String> {
    let surface = surface.rotozoom(0.0, 1.0, false)?;
    let mut surface = surface.into_canvas()?;
    let surface_texture_creator = surface.texture_creator();
    let texture = surface_texture_creator.load_texture(Path::new("assets/tile.png"))?;
    surface.copy(&texture, rect!(0, 0, 64, 64), rect!(0, 0, 128, 128))?;
    let surface = surface.into_surface();

    Ok(surface)
}

fn draw_tachometer_foreground<'a>(
    surface: &'a Surface<'a>,
    angle: f64,
) -> Result<Surface<'_>, String> {
    let surface = surface.rotozoom(-angle, 1.0, true)?;
    let mut surface = surface.into_canvas()?;
    let surface_texture_creator = surface.texture_creator();
    let texture = surface_texture_creator.load_texture(Path::new("assets/tile.png"))?;
    surface.copy(&texture, rect!(64, 0, 64, 64), rect!(0, 0, 128, 128))?;
    let surface = surface.into_surface();

    Ok(surface)
}

fn draw_tachometer(
    canvas: &mut WindowCanvas,
    texture_creator: &TextureCreator<WindowContext>,
    position: (usize, usize),
    angle: f64,
) -> Result<(), String> {
    let surface = Surface::new(128, 128, PixelFormatEnum::RGB24)?;
    let surface = draw_tachometer_background(&surface)?;
    let surface = draw_tachometer_foreground(&surface, angle)?;
    let surface = surface.rotozoom(angle, 1.0, false)?;

    let texture = surface
        .as_texture(&texture_creator)
        .map_err(|e| e.to_string())?;

    canvas.copy(&texture, None, rect!(position.0, position.1, 128, 128))?;

    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let window = prepare_window(&sdl_context)?;
    let mut canvas = prepare_canvas(window)?;

    let mut angle = 0.0;
    let texture_creator = canvas.texture_creator();

    'game_loop: loop {
        canvas.set_draw_color(Color::RGB(1, 25, 54));
        canvas.clear();

        draw_tachometer(&mut canvas, &texture_creator, (64 * 1, 64 * 7), angle)?;

        canvas.present();
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'game_loop Ok(()),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => angle += 1.0,
                _ => (),
            }
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
