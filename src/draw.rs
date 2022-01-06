use sdl2::{
	rect, render,
	video::{FullscreenType, Window},
};

use crate::{
	config::{Font, Rgb, ThemeConfig},
	font, m8,
};

pub const LINE_HEIGHT: i32 = 10;

pub struct Context<'a, 'b, 'c> {
	pub canvas: &'b mut render::Canvas<Window>,
	pub font: &'a mut render::Texture<'c>,
	pub font_option: Font,
	pub theme: ThemeConfig,
	pub screen_bg: Option<Rgb>,
}

impl<'a, 'b, 'c> Context<'_, '_, '_> {
	pub fn draw_str(&mut self, s: &str, x: i32, y: i32, fg: Rgb, bg: Rgb) -> Result<(), String> {
		let mut x = x;
		for ch in s.chars() {
			self.draw_char(ch as u8, x, y, fg, bg)?;
			x += font::CHAR_WIDTH as i32;
		}
		Ok(())
	}

	pub fn draw_char(&mut self, c: u8, x: i32, y: i32, fg: Rgb, bg: Rgb) -> Result<(), String> {
		let c = match self.font_option {
			Font::UpperAltZero if c == b'0' => 125,
			Font::LowerAltZero if c == b'0' => 123,
			Font::Uppercase | Font::UpperAltZero => {
				if (b'a'..=b'z').contains(&c) {
					c - 32
				} else {
					c
				}
			}
			Font::Lowercase | Font::LowerAltZero => {
				if (b'A'..=b'Z').contains(&c) {
					c + 32
				} else {
					c
				}
			}
		};
		let row = c as i32 / font::CHARS_BY_ROW;
		let col = c as i32 % font::CHARS_BY_ROW;
		let src_rect = rect::Rect::new(
			col * font::CHAR_WIDTH,
			row * font::CHAR_HEIGHT,
			font::CHAR_WIDTH as u32,
			font::CHAR_HEIGHT as u32,
		);
		let dst_rect = rect::Rect::new(x, y + 3, src_rect.w as u32, src_rect.h as u32);
		self.font.set_color_mod(fg.0, fg.1, fg.2);
		if fg != bg {
			let bg_rect = rect::Rect::new(
				x as i32 - 1,
				y as i32 + 2,
				font::CHAR_WIDTH as u32 - 1,
				font::CHAR_HEIGHT as u32 + 1,
			);
			self.canvas.set_draw_color(bg.rgb());
			self.canvas.fill_rect(bg_rect)?;
		}
		self.canvas.copy(self.font, src_rect, dst_rect)
	}

	pub fn draw_rect(&mut self, rect: (i32, i32, u32, u32), bg: Rgb) -> Result<(), String> {
		let r = rect::Rect::new(rect.0, rect.1, rect.2, rect.3);
		if rect.0 == 0 && rect.1 == 0 && rect.2 == m8::SCREEN_WIDTH && rect.3 == m8::SCREEN_HEIGHT {
			self.screen_bg = Some(bg);
		}
		self.canvas.set_draw_color(bg.rgb());
		self.canvas.fill_rect(r)
	}

	pub fn draw_waveform(&mut self, data: &[u8], fg: (u8, u8, u8)) -> Result<(), String> {
		self.canvas.set_draw_color(self.theme.screen.rgb());
		let rect = rect::Rect::new(0, 0, m8::SCREEN_WIDTH, m8::WAVEFORM_HEIGHT);
		self.canvas.fill_rect(rect)?;
		if data.is_empty() {
			return Ok(());
		}
		self.canvas.set_draw_color(fg);

		let mut points = [rect::Point::new(0, 0); m8::SCREEN_WIDTH as usize];
		for (i, p) in data.iter().enumerate() {
			points[i] = rect::Point::new(i as i32, *p as i32);
		}
		self.canvas.draw_points(points.as_ref())
	}

	pub fn draw_octave(&mut self, octave: u8, show: bool) -> Result<(), String> {
		let x = m8::SCREEN_WIDTH as i32 - font::CHAR_WIDTH;
		let y = m8::SCREEN_HEIGHT as i32 - font::CHAR_HEIGHT;

		let rect = rect::Rect::new(
			x + 1,
			y - 1,
			font::CHAR_WIDTH as u32 - 1,
			font::CHAR_HEIGHT as u32 + 1,
		);
		self.canvas.set_draw_color(self.theme.screen.rgb());
		self.canvas.fill_rect(rect)?;

		if show {
			let c = if octave >= 9 { octave - 9 + b'A' } else { octave + b'1' };
			let x = x - 1;
			let y = y - 3;
			let fg = self.theme.octave_fg;
			let bg = self.theme.octave_bg;
			self.draw_char(c, x + 3, y, fg, bg)?;
		}
		Ok(())
	}

	pub fn draw_velocity(&mut self, velocity: u8, show: bool) -> Result<(), String> {
		let mut x = m8::SCREEN_WIDTH as i32 - font::CHAR_WIDTH * 3 + 2;
		let y = m8::SCREEN_HEIGHT as i32 - font::CHAR_HEIGHT;

		let rect = rect::Rect::new(
			x as i32 - 1,
			y as i32 - 1,
			font::CHAR_WIDTH as u32 * 2 - 2,
			font::CHAR_HEIGHT as u32 + 1,
		);
		self.canvas.set_draw_color(self.theme.screen.rgb());
		self.canvas.fill_rect(rect)?;

		if show {
			let (vh, vl) = (velocity >> 4, velocity & 0xf);
			let fg = self.theme.velocity_fg;
			let bg = self.theme.velocity_bg;
			let c1 = if vh > 9 { vh - 10 + b'A' } else { vh + b'0' };
			self.draw_char(c1, x, y - 3, fg, bg)?;
			x += font::CHAR_WIDTH - 1;
			let c2 = if vl > 9 { vl - 10 + b'A' } else { vl + b'0' };
			self.draw_char(c2, x, y - 3, fg, bg)?;
		}
		Ok(())
	}
}

pub fn is_fullscreen(canvas: &render::Canvas<Window>) -> bool {
	!matches!(canvas.window().fullscreen_state(), FullscreenType::Off)
}

pub fn toggle_fullscreen(canvas: &mut render::Canvas<Window>) -> Result<(), String> {
	match canvas.window().fullscreen_state() {
		FullscreenType::Off => canvas.window_mut().set_fullscreen(FullscreenType::True),
		_ => canvas.window_mut().set_fullscreen(FullscreenType::Off),
	}
}

pub fn zoom_window(window: &mut Window, zoom: u32) {
	let (w, _) = window.size();
	if m8::SCREEN_WIDTH * zoom != w {
		let _ = window.set_size(zoom * m8::SCREEN_WIDTH, zoom * m8::SCREEN_HEIGHT);
	}
}
