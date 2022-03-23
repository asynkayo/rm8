#[macro_use]
extern crate serde_derive;

use sdl2::{
	event::Event,
	keyboard::{Keycode, Mod},
	pixels::PixelFormatEnum,
	video,
};
use std::sync::{
	atomic::{self, AtomicBool},
	Arc,
};

mod app;
mod audio;
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
	let running = Arc::new(AtomicBool::new(true));
	let ctrlc_running = running.clone();
	let mut app = App::new(running);

	ctrlc::set_handler(move || ctrlc_running.store(false, atomic::Ordering::SeqCst))
		.map_err(|e| e.to_string())?;

	// process command line arguments
	let mut device: Option<String> = None;
	let mut config_file: Option<String> = None;
	let mut capture: Option<String> = None;
	if !cli::handle_command_line(app.config_mut(), &mut config_file, &mut device, &mut capture)? {
		return Ok(());
	}
	// detect and connect to M8
	let mut m8 = match device {
		Some(dev) => M8::open(dev),
		None => M8::detect(),
	}
	.map_err(|e| e.to_string())?;
	m8.set_reconnect(app.config().app.reconnect);
	m8.enable_and_reset_display()?;
	m8.keyjazz.set(!app.config().overlap);

	app.build_menu(&m8);

	let sdl_context = sdl2::init()?;
	let joystick_subsystem = sdl_context.joystick()?;
	let video_subsystem = sdl_context.video()?;
	let audio_subsystem = sdl_context.audio()?;
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

	m8.connect_audio(audio::Audio::open(&audio_subsystem, capture)?);

	let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
	canvas.set_logical_size(m8::SCREEN_WIDTH, m8::SCREEN_HEIGHT).map_err(|e| e.to_string())?;

	let creator = canvas.texture_creator();
	let mut texture = creator
		.create_texture_target(PixelFormatEnum::ARGB8888, m8::SCREEN_WIDTH, m8::SCREEN_HEIGHT)
		.map_err(|e| e.to_string())?;

	let mut font = font::init(&creator)?;

	let mut event_pump = sdl_context.event_pump()?;
	while app.running() {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } => {
					app.quit();
					continue;
				}
				Event::KeyDown { keycode: Some(keycode), keymod, repeat, .. } => {
					if keycode == Keycode::Escape {
						app.escape_command(&mut m8, &mut canvas)?;
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
							app.action_modified(&mut canvas, &mut m8, &joystick_subsystem)?;
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
		app.handle_defer(&mut m8, &mut canvas)?;
		if app.sync() {
			if app.config_mode() {
				app.process_action(&mut canvas, &mut m8, &joystick_subsystem, &config_file)?;

				canvas
					.with_texture_canvas(&mut texture, |target| {
						let config = app.config();
						let ctx = &mut draw::Context {
							canvas: target,
							font: &mut font,
							theme: config.theme,
							font_option: config.app.font,
							screen_bg: None,
						};
						let _ = app.render(ctx);
						let _ = app.render_fps(ctx);
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
					.with_texture_canvas(&mut texture, |target| {
						let config = app.config();
						let ctx = &mut draw::Context {
							canvas: target,
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
						if m8.disconnected() {
							let _ = ctx.clear();
							let fg = ctx.theme.text_info;
							let _ = ctx.draw_str_centered(
								"M8 LOST",
								m8::SCREEN_HEIGHT as i32 / 2,
								fg,
								fg,
							);
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
						let _ = app.render_fps(ctx);
					})
					.map_err(|e| e.to_string())?;
			}

			canvas.set_draw_color(app.config().theme.screen.rgb());
			canvas.clear();
			canvas.copy(&texture, None, None)?;
			canvas.present();
		}
	}

	Ok(())
}
