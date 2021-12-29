#[macro_use]
extern crate serde_derive;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod, Scancode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::video;
use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

mod font;
mod m8;
mod slip;
mod value;
use m8::{Command, M8};
mod config;
use config::{Config, Remap};
mod cli;
mod display;
use display::Display;

fn main() -> Result<(), String> {
	let mut config = Config::default();
	let _ = config.read(config::FILE);

	// process command line arguments
	let mut device: Option<String> = None;
	let mut config_file: Option<String> = None;
	if let cli::Action::Return =
		cli::handle_command_line(&mut config, &mut config_file, &mut device)?
	{
		return Ok(());
	}

	let running = Arc::new(AtomicBool::new(true));
	let run = running.clone();
	ctrlc::set_handler(move || run.store(false, Ordering::SeqCst)).map_err(|e| e.to_string())?;

	// initialize M8 display helpers
	let mut display = Display::new();
	// detect and connect to M8
	let mut m8 = match device {
		Some(dev) => M8::open(dev),
		None => M8::detect(),
	}
	.map_err(|e| e.to_string())?;
	m8.enable_and_reset_display()?;
	m8.keyjazz.set(!config.overlap);

	// initialize SDL
	let sdl_context = sdl2::init()?;
	let video_subsystem = sdl_context.video()?;
	let mut window = video_subsystem
		.window("rm8", m8::SCREEN_WIDTH * 4, m8::SCREEN_HEIGHT * 4)
		.position_centered()
		.opengl()
		.resizable()
		.build()
		.map_err(|e| e.to_string())?;
	window.set_fullscreen(if config.fullscreen {
		video::FullscreenType::True
	} else {
		video::FullscreenType::Off
	})?;

	let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
	canvas.set_logical_size(m8::SCREEN_WIDTH, m8::SCREEN_HEIGHT).map_err(|e| e.to_string())?;

	let creator = canvas.texture_creator();
	let mut texture = creator
		.create_texture_target(PixelFormatEnum::ARGB8888, m8::SCREEN_WIDTH, m8::SCREEN_HEIGHT)
		.map_err(|e| e.to_string())?;

	let mut font = font::init(&creator)?;

	let mut remap: Option<Remap> = None;

	let mut event_pump = sdl_context.event_pump()?;
	while running.load(Ordering::SeqCst) {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } => return Ok(()),
				Event::KeyDown {
					keycode: Some(keycode),
					scancode: Some(scancode),
					keymod: mods,
					..
				} => {
					if let Some(ref mut remapping) = remap {
						match keycode {
							Keycode::Escape => remap = None,
							_ => {
								if remapping.done() {
									continue;
								}
								remapping.map(scancode);
								if remapping.done() {
									remapping.write(&mut config);
									config.write(config_file.as_deref().unwrap_or(config::FILE))?;
								}
							}
						}
						continue;
					}
					if scancode == Scancode::Escape {
						if display.is_fullscreen(&canvas) {
							display.toggle_fullscreen(&mut canvas)?;
						} else {
							running.store(false, Ordering::SeqCst);
						}
						continue;
					} else if mods.intersects(Mod::LALTMOD | Mod::RALTMOD)
						&& scancode == Scancode::Return
					{
						display.toggle_fullscreen(&mut canvas)?;
					} else if mods.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD)
						&& keycode == Keycode::R
					{
						remap = Some(Remap::new());
						m8.reset_display()?;
						continue;
					} else if mods.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD)
						&& keycode == Keycode::R
					{
						m8.reset(mods.intersects(Mod::LALTMOD | Mod::RALTMOD))?;
						continue;
					} else if scancode == *config.keyjazz {
						m8.keyjazz.toggle();
					}
					if !config.overlap || *m8.keyjazz {
						config.handle_keyjazz(
							&mut m8,
							scancode,
							mods.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD),
						);
					}
					if !config.overlap || !*m8.keyjazz {
						config.handle_keys(&mut m8, scancode, true);
					}
				}
				Event::KeyUp { scancode: Some(scancode), .. } => {
					if remap.is_some() {
						continue;
					}
					if config::KEYJAZZ.contains(&scancode) {
						m8.set_note_off()
					} else {
						config.handle_keys(&mut m8, scancode, false);
					}
				}
				_ => {}
			}
		}

		if m8.note.changed() {
			m8.send_keyjazz()?;
		}
		if m8.keys.changed() {
			m8.send_keys()?;
		}

		canvas
			.with_texture_canvas(&mut texture, |mut texture_canvas| {
				if let Some(ref remapping) = remap {
					let _ = display.draw_mapping(&mut texture_canvas, &mut font, remapping);
					return;
				}
				while let Ok(Some(cmd)) = m8.read() {
					let _ = match cmd {
						Command::Joypad { .. } => Ok(()),
						Command::Waveform(fg, data) => {
							display.draw_waveform(&mut texture_canvas, data, fg)
						}
						Command::Character(c, x, y, fg, bg) => {
							display.draw_character(&mut texture_canvas, &mut font, c, x, y, fg, bg)
						}
						Command::Rectangle(x, y, w, h, bg) => {
							display.draw_rectangle(&mut texture_canvas, x, y, w, h, bg)
						}
					};
				}
				let (kc, vc, oc) =
					(m8.keyjazz.changed(), m8.velocity.changed(), m8.octave.changed());
				if kc || vc {
					let _ = display.draw_velocity(
						&mut texture_canvas,
						&mut font,
						*m8.velocity,
						*m8.keyjazz,
					);
				}
				if kc || oc {
					let _ = display.draw_octave(
						&mut texture_canvas,
						&mut font,
						*m8.octave,
						*m8.keyjazz,
					);
				}
			})
			.map_err(|e| e.to_string())?;

		let now = std::time::Instant::now();
		if now - display.ticks > Duration::from_millis(15) {
			display.ticks = now;
			canvas.set_draw_color(display.bg);
			canvas.clear();
			canvas.copy(&texture, None, None)?;
			canvas.present();
		} else {
			std::thread::sleep(Duration::from_millis(0));
		}
	}
	Ok(())
}
