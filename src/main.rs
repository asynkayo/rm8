#[macro_use]
extern crate serde_derive;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod, Scancode};
use sdl2::pixels::{self, PixelFormatEnum};
use sdl2::{rect, render, video};
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

struct Display {
	bg: pixels::Color,
	ticks: std::time::Instant,
}

impl Display {
	fn new() -> Self {
		let now = std::time::Instant::now();
		Self { bg: pixels::Color::RGB(0, 0, 0), ticks: now }
	}

	fn draw_waveform(
		&self,
		canvas: &mut render::Canvas<video::Window>,
		data: &[u8],
		fg: (u8, u8, u8),
	) -> Result<(), String> {
		canvas.set_draw_color(self.bg);
		let rect = rect::Rect::new(0, 0, m8::SCREEN_WIDTH, m8::WAVEFORM_HEIGHT);
		canvas.fill_rect(rect)?;
		if data.is_empty() {
			return Ok(());
		}
		canvas.set_draw_color(fg);

		let mut points = [rect::Point::new(0, 0); m8::SCREEN_WIDTH as usize];
		for (i, p) in data.iter().enumerate() {
			points[i] = rect::Point::new(i as i32, *p as i32);
		}
		canvas.draw_points(points.as_ref())
	}

	fn draw_rectangle(
		&mut self,
		canvas: &mut render::Canvas<video::Window>,
		x: u16,
		y: u16,
		w: u16,
		h: u16,
		bg: (u8, u8, u8),
	) -> Result<(), String> {
		let rect = rect::Rect::new(x as i32, y as i32, w as u32, h as u32);
		if x == 0 && y == 0 && w == m8::SCREEN_WIDTH as u16 && h == m8::SCREEN_HEIGHT as u16 {
			self.bg = pixels::Color::RGB(bg.0, bg.1, bg.2);
		}
		canvas.set_draw_color(bg);
		canvas.fill_rect(rect)
	}

	fn draw_octave<'a>(
		&self,
		canvas: &mut render::Canvas<video::Window>,
		font: &mut render::Texture<'a>,
		octave: u8,
		show: bool,
	) -> Result<(), String> {
		let x = m8::SCREEN_WIDTH - font::CHAR_WIDTH;
		let y = m8::SCREEN_HEIGHT - font::CHAR_HEIGHT;

		let rect = rect::Rect::new(
			x as i32 + 1,
			y as i32 - 1,
			font::CHAR_WIDTH - 1,
			font::CHAR_HEIGHT + 1,
		);
		canvas.set_draw_color(self.bg);
		canvas.fill_rect(rect)?;

		if show {
			let c = if octave >= 9 { octave - 9 + b'A' } else { octave + b'1' };
			let x = x as u16 - 1;
			let y = y as u16 - 3;
			let fg = (0xff, 0xff, 0xff);
			let bg = (0, 0, 0xff);
			self.draw_character(canvas, font, c, x + 3, y, fg, bg)?;
		}
		Ok(())
	}

	fn draw_velocity<'a>(
		&self,
		canvas: &mut render::Canvas<video::Window>,
		font: &mut render::Texture<'a>,
		velocity: u8,
		show: bool,
	) -> Result<(), String> {
		let mut x = m8::SCREEN_WIDTH - font::CHAR_WIDTH * 3 + 2;
		let y = m8::SCREEN_HEIGHT - font::CHAR_HEIGHT;

		let rect = rect::Rect::new(
			x as i32 - 1,
			y as i32 - 1,
			font::CHAR_WIDTH * 2 - 2,
			font::CHAR_HEIGHT + 1,
		);
		canvas.set_draw_color(self.bg);
		canvas.fill_rect(rect)?;

		if show {
			let (vh, vl) = (velocity >> 4, velocity & 0xf);
			let fg = (0xff, 0xff, 0xff);
			let bg = (0xff, 0, 0);
			let c1 = if vh > 9 { vh - 10 + b'A' } else { vh + b'0' };
			self.draw_character(canvas, font, c1, x as u16, y as u16 - 3, fg, bg)?;
			x += font::CHAR_WIDTH - 1;
			let c2 = if vl > 9 { vl - 10 + b'A' } else { vl + b'0' };
			self.draw_character(canvas, font, c2, x as u16, y as u16 - 3, fg, bg)?;
		}
		Ok(())
	}

	fn draw_string_centered<'a>(
		&self,
		canvas: &mut render::Canvas<video::Window>,
		font: &mut render::Texture<'a>,
		s: &str,
		y: u16,
		fg: (u8, u8, u8),
		bg: (u8, u8, u8),
	) -> Result<(), String> {
		let x = m8::SCREEN_WIDTH as u16 / 2 - s.len() as u16 * font::CHAR_WIDTH as u16 / 2 + 1;
		self.draw_string(canvas, font, s, x, y, fg, bg)
	}

	#[allow(clippy::too_many_arguments)]
	fn draw_string<'a>(
		&self,
		canvas: &mut render::Canvas<video::Window>,
		font: &mut render::Texture<'a>,
		s: &str,
		x: u16,
		y: u16,
		fg: (u8, u8, u8),
		bg: (u8, u8, u8),
	) -> Result<(), String> {
		let mut x = x;
		for ch in s.chars() {
			self.draw_character(canvas, font, ch as u8, x, y, fg, bg)?;
			x += font::CHAR_WIDTH as u16;
		}
		Ok(())
	}

	#[allow(clippy::too_many_arguments)]
	fn draw_character<'a>(
		&self,
		canvas: &mut render::Canvas<video::Window>,
		font: &mut render::Texture<'a>,
		c: u8,
		x: u16,
		y: u16,
		fg: (u8, u8, u8),
		bg: (u8, u8, u8),
	) -> Result<(), String> {
		let row = c as u32 / font::CHARS_BY_ROW;
		let col = c as u32 % font::CHARS_BY_ROW;
		let src_rect = rect::Rect::new(
			(col * font::CHAR_WIDTH) as i32,
			(row * font::CHAR_HEIGHT) as i32,
			font::CHAR_WIDTH,
			font::CHAR_HEIGHT,
		);
		let dst_rect =
			rect::Rect::new(x as i32, y as i32 + 3, src_rect.w as u32, src_rect.h as u32);
		font.set_color_mod(fg.0, fg.1, fg.2);
		if fg != bg {
			let bg_rect = rect::Rect::new(
				x as i32 - 1,
				y as i32 + 2,
				font::CHAR_WIDTH - 1,
				font::CHAR_HEIGHT + 1,
			);
			canvas.set_draw_color(bg);
			canvas.fill_rect(bg_rect)?;
		}
		canvas.copy(font, src_rect, dst_rect)
	}

	fn draw_mapping<'a>(
		&mut self,
		canvas: &mut render::Canvas<video::Window>,
		font: &mut render::Texture<'a>,
		remapping: &Remap,
	) -> Result<(), String> {
		if remapping.init {
			canvas.set_draw_color(self.bg);
			canvas.clear();
			let y1 = m8::SCREEN_HEIGHT / 3;
			let color1 = (0x32, 0xec, 0xff);
			self.draw_string_centered(canvas, font, "CONFIG", y1 as u16, color1, color1)?;
			let y2 = m8::SCREEN_HEIGHT / 4 * 3;
			let color2 = (0x32, 0xec, 0xff);
			self.draw_string_centered(canvas, font, "ESC = ABORT", y2 as u16, color2, color2)?;
		}

		let color = (0x60, 0x60, 0x8e);
		let y1 = m8::SCREEN_HEIGHT / 2 - font::CHAR_HEIGHT;
		let y2 = m8::SCREEN_HEIGHT / 2 + font::CHAR_HEIGHT;
		let rect = rect::Rect::new(0, y1 as i32 + 6, m8::SCREEN_WIDTH, y2 + font::CHAR_HEIGHT - y1);
		canvas.set_draw_color(self.bg);
		canvas.fill_rect(rect)?;
		if remapping.done() {
			self.draw_string_centered(canvas, font, "- DONE -", y1 as u16, color, color)?;
		} else {
			let map = remapping.current();
			self.draw_string_centered(canvas, font, map, y1 as u16, color, color)?;
		}

		if remapping.exists {
			let color = (0xff, 0x30, 0x70);
			self.draw_string_centered(canvas, font, "MAPPING EXISTS!", y2 as u16, color, color)?;
		}
		Ok(())
	}

	fn is_fullscreen(&self, canvas: &render::Canvas<video::Window>) -> bool {
		!matches!(canvas.window().fullscreen_state(), video::FullscreenType::Off)
	}

	fn toggle_fullscreen(&self, canvas: &mut render::Canvas<video::Window>) -> Result<(), String> {
		match canvas.window().fullscreen_state() {
			video::FullscreenType::Off => {
				canvas.window_mut().set_fullscreen(video::FullscreenType::True)
			}
			_ => canvas.window_mut().set_fullscreen(video::FullscreenType::Off),
		}
	}
}

fn main() -> Result<(), String> {
	let mut config = Config::default();
	let _ = config.read(config::FILE);

	// process command line arguments
	let mut device: Option<String> = None;
	let mut config_file: Option<String> = None;
	if let cli::Action::Return = cli::handle_command_line(&mut config, &mut config_file, &mut device)? {
		return Ok(());
	}

	let running = Arc::new(AtomicBool::new(true));
	let run = running.clone();
	ctrlc::set_handler(move || run.store(false, Ordering::SeqCst)).map_err(|e| e.to_string())?;

	// detect and connect to M8
	let mut m8 = match device {
		Some(dev) => M8::open(dev),
		None => M8::detect(),
	}
	.map_err(|e| e.to_string())?;
	m8.enable_and_reset_display()?;
	m8.keyjazz.set(!config.overlap);

	// initialize M8 display helpers
	let mut display = Display::new();

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
	// create display texture
	let mut texture = creator
		.create_texture_target(PixelFormatEnum::ARGB8888, m8::SCREEN_WIDTH, m8::SCREEN_HEIGHT)
		.map_err(|e| e.to_string())?;

	// prepare font texture
	let mut font = font::init(&creator)?;

	// key remapper
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
