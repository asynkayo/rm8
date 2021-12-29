use std::time;

use sdl2::pixels;
use sdl2::{rect, render, video};

use crate::m8;
use crate::font;
use crate::config::Remap;

pub struct Display {
	pub bg: pixels::Color,
	pub ticks: time::Instant,
}

impl Display {
	pub fn new() -> Self {
		let now = time::Instant::now();
		Self { bg: pixels::Color::RGB(0, 0, 0), ticks: now }
	}

	pub fn draw_waveform(
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

	pub fn draw_rectangle(
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

	pub fn draw_octave<'a>(
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

	pub fn draw_velocity<'a>(
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

	pub fn draw_string_centered<'a>(
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
	pub fn draw_string<'a>(
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
	pub fn draw_character<'a>(
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

	pub fn draw_mapping<'a>(
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

	pub fn is_fullscreen(&self, canvas: &render::Canvas<video::Window>) -> bool {
		!matches!(canvas.window().fullscreen_state(), video::FullscreenType::Off)
	}

	pub fn toggle_fullscreen(&self, canvas: &mut render::Canvas<video::Window>) -> Result<(), String> {
		match canvas.window().fullscreen_state() {
			video::FullscreenType::Off => {
				canvas.window_mut().set_fullscreen(video::FullscreenType::True)
			}
			_ => canvas.window_mut().set_fullscreen(video::FullscreenType::Off),
		}
	}
}
