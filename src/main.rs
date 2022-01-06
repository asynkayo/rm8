#[macro_use]
extern crate serde_derive;

use sdl2::{
	event::Event,
	keyboard::{Keycode, Mod},
	pixels::PixelFormatEnum,
	video,
};
use std::{
	sync::{
		atomic::{self, AtomicBool},
		Arc,
	},
	thread,
	time::Duration,
};

mod app;
mod cli;
mod config;
mod config_command;
mod config_font;
mod config_joystick;
mod config_keycode;
mod config_rgb;
mod draw;
mod font;
mod m8;
mod menu;
mod menu_tools;
mod nav;
mod nav_item;
mod nav_page;
mod remap;
mod slip;
mod value;

use app::App;
use config::Rgb;
use m8::M8;

fn main() -> Result<(), String> {
	let mut app = App::new();
	let _ = app.config_mut().read(app::CONFIG_FILE);

	let running = Arc::new(AtomicBool::new(true));
	let run = running.clone();

	ctrlc::set_handler(move || run.store(false, atomic::Ordering::SeqCst))
		.map_err(|e| e.to_string())?;

	// process command line arguments
	let mut device: Option<String> = None;
	let mut config_file: Option<String> = None;
	if !cli::handle_command_line(app.config_mut(), &mut config_file, &mut device)? {
		return Ok(());
	}

	// detect and connect to M8
	let mut m8 = match device {
		Some(dev) => M8::open(dev),
		None => M8::detect(),
	}
	.map_err(|e| e.to_string())?;
	m8.enable_and_reset_display()?;
	m8.keyjazz.set(!app.config().overlap);

	let sdl_context = sdl2::init()?;
	let joystick_subsystem = sdl_context.joystick()?;
	let video_subsystem = sdl_context.video()?;
	let zoom = app.config().app.zoom;
	let mut window = video_subsystem
		.window("rm8", m8::SCREEN_WIDTH * zoom, m8::SCREEN_HEIGHT * zoom)
		.position_centered()
		.opengl()
		.resizable()
		.build()
		.map_err(|e| e.to_string())?;
	window.set_fullscreen(if app.config().app.fullscreen {
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

	let mut event_pump = sdl_context.event_pump()?;
	while running.load(atomic::Ordering::SeqCst) {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } => {
					running.store(false, atomic::Ordering::SeqCst);
					continue;
				}
				Event::KeyDown { keycode: Some(keycode), keymod, repeat, .. } => {
					if keycode == Keycode::Escape {
						if app.config_mode() {
							if app.remap_mode() {
								app.cancel_remap_mode();
								continue;
							}
							app.cancel_config_mode();
							m8.refresh();
						} else if draw::is_fullscreen(&canvas) {
							draw::toggle_fullscreen(&mut canvas)?;
						} else {
							running.store(false, atomic::Ordering::SeqCst);
						}
						continue;
					}
					if repeat || app.remap_mode() {
						continue;
					}
					if keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) {
						match keycode {
							Keycode::Return => {
								draw::toggle_fullscreen(&mut canvas)?;
								continue;
							}
							Keycode::C if !app.config_mode() => {
								app.start_config_mode();
								m8.reset_display()?;
								continue;
							}
							Keycode::R if !app.config_mode() => {
								m8.reset(keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD))?;
								continue;
							}
							_ => {}
						}
					}

					if !app.config_mode() {
						let config = app.config();
						if keycode == *config.rm8.keyjazz {
							m8.keyjazz.toggle();
						}
						if !config.overlap || *m8.keyjazz {
							if let Some(n) = config.keyjazz.get(&keycode.into()) {
								m8.set_note(*n);
							}
						}
					}
					app.handle_key(&mut m8, keycode, keymod, false);
				}
				Event::KeyUp { keycode: Some(keycode), keymod, .. } => {
					if app.remap_mode() {
						if app.remap(keycode) {
							app.cancel_remap_mode();
						}
						continue;
					}
					app.handle_key(&mut m8, keycode, keymod, true);
				}
				Event::JoyAxisMotion { which, axis_idx, value, .. } => {
					app.handle_cmd(&mut m8, app.axis_cmd(which, axis_idx, value));
				}
				Event::JoyHatMotion { which, state, .. } => {
					app.handle_cmd(&mut m8, app.hat_cmd(which, state));
				}
				Event::JoyButtonDown { which, button_idx, .. } => {
					app.handle_cmd(&mut m8, app.button_cmd(which, button_idx, false));
				}
				Event::JoyButtonUp { which, button_idx, .. } => {
					app.handle_cmd(&mut m8, app.button_cmd(which, button_idx, true));
				}
				Event::JoyDeviceAdded { which, .. } => {
					app.add_joystick(&joystick_subsystem, which);
				}
				Event::JoyDeviceRemoved { which, .. } => {
					app.rem_joystick(&joystick_subsystem, which)
				}
				_ => (),
			}
		}

		app.process_key(&mut m8);
		if app.config_mode() {
			app.process_action(&mut canvas, &joystick_subsystem, &config_file)?;

			canvas
				.with_texture_canvas(&mut texture, |mut target| {
					let config = app.config();
					let ctx = &mut draw::Context {
						canvas: &mut target,
						font: &mut font,
						theme: config.theme,
						font_option: config.app.font,
						screen_bg: None,
					};
					let _ = app.render(ctx);
				})
				.map_err(|e| e.to_string())?;
		} else {
			if m8.note.changed() {
				m8.send_keyjazz()?;
			}
			if m8.keys.changed() {
				m8.send_keys()?;
			}

			canvas
				.with_texture_canvas(&mut texture, |mut target| {
					let config = app.config();
					let ctx = &mut draw::Context {
						canvas: &mut target,
						font: &mut font,
						theme: config.theme,
						font_option: config.app.font,
						screen_bg: None,
					};
					while let Ok(Some(cmd)) = m8.read() {
						let _ = match cmd {
							m8::Command::Joypad { .. } => Ok(()),
							m8::Command::Waveform(fg, data) => ctx.draw_waveform(data, fg),
							m8::Command::Character(c, x, y, fg, bg) => ctx.draw_char(
								c,
								x as i32,
								y as i32,
								Rgb::from_tuple(fg),
								Rgb::from_tuple(bg),
							),
							m8::Command::Rectangle(x, y, w, h, bg) => ctx.draw_rect(
								(x as i32, y as i32, w as u32, h as u32),
								Rgb::from_tuple(bg),
							),
						};
					}
					let (kc, vc, oc) =
						(m8.keyjazz.changed(), m8.velocity.changed(), m8.octave.changed());
					if kc || vc {
						let _ = ctx.draw_velocity(*m8.velocity, *m8.keyjazz);
					}
					if kc || oc {
						let _ = ctx.draw_octave(*m8.octave, *m8.keyjazz);
					}
					if let Some(bg) = ctx.screen_bg {
						app.config_mut().theme.screen = bg;
					}
				})
				.map_err(|e| e.to_string())?;
		}

		if app.sync() {
			canvas.set_draw_color(app.config().theme.screen.rgb());
			canvas.clear();
			canvas.copy(&texture, None, None)?;
			canvas.present();
		} else {
			thread::sleep(Duration::from_millis(10));
		}
	}

	Ok(())
}
