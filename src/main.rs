#[macro_use]
extern crate serde_derive;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod, Scancode};
use sdl2::pixels::{self, PixelFormatEnum};
use sdl2::{
	rect,
	render::{self, BlendMode, TextureAccess},
	video,
};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serialport::{available_ports, ErrorKind, SerialPort, SerialPortType};
use std::{
	io::{self, Write},
	mem,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

const SLIP_END: u8 = 0xc0;
const SLIP_ESC: u8 = 0xdb;
const SLIP_ESC_END: u8 = 0xdc;
const SLIP_ESC_ESC: u8 = 0xdd;

enum SlipState {
	Normal,
	Escape,
}

struct Slip<const N: usize> {
	state: SlipState,
	buf: [u8; N],
	rmax: usize,
	rpos: usize,
	wpos: usize,
}

impl<const N: usize> Slip<N> {
	fn new() -> Self {
		Self { state: SlipState::Normal, buf: [0; N], rmax: 0, rpos: 0, wpos: 0 }
	}

	fn push_byte(&mut self, byte: u8, buf: &mut [u8]) -> Result<(), String> {
		if self.wpos >= buf.len() {
			self.wpos = 0;
			return Err("push_byte overflow".to_string());
		}
		buf[self.wpos] = byte;
		self.wpos += 1;
		Ok(())
	}

	fn read<'a>(
		&mut self,
		port: &mut Box<dyn SerialPort>,
		buf: &'a mut [u8],
	) -> Result<Option<&'a [u8]>, String> {
		loop {
			if self.rpos >= self.rmax {
				self.rpos = 0;
				match port.read(&mut self.buf) {
					Ok(n) => self.rmax = n,
					Err(e) if e.kind() == io::ErrorKind::TimedOut => self.rmax = 0,
					Err(e) => return Err(e.to_string()),
				}
				if self.rmax == 0 {
					return Ok(None);
				}
			}
			while self.rpos < self.rmax {
				let byte = self.buf[self.rpos];
				match self.state {
					SlipState::Normal => match byte {
						SLIP_END if self.wpos > 1 => {
							self.rpos += 1;
							let end = self.wpos;
							self.wpos = 0;
							return Ok(Some(&buf[..end]));
						}
						SLIP_END => return Err("empty command".to_string()),
						SLIP_ESC => {
							self.state = SlipState::Escape;
							self.rpos += 1;
							continue;
						}
						_ => self.push_byte(byte, buf)?,
					},
					SlipState::Escape => match byte {
						SLIP_ESC_END => self.push_byte(SLIP_END, buf)?,
						SLIP_ESC_ESC => self.push_byte(SLIP_ESC, buf)?,
						_ => return Err(format!("invalid escape sequence: {:02x}", byte)),
					},
				}
				self.state = SlipState::Normal;
				self.rpos += 1;
			}
		}
	}
}

struct Value<T> {
	value: T,
	modified: bool,
}

impl<T> std::ops::Deref for Value<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T: std::cmp::Eq + Copy> Value<T> {
	fn new(value: T) -> Self {
		Self { value, modified: false }
	}

	fn set(&mut self, value: T) {
		if self.value != value {
			self.value = value;
			self.modified = true;
		}
	}

	fn changed(&mut self) -> bool {
		if self.modified {
			self.modified = false;
			return true;
		}
		false
	}
}

impl Value<bool> {
	fn toggle(&mut self) {
		self.value = !self.value;
		self.modified = true;
	}
}

impl Value<u8> {
	fn set_bit(&mut self, mask: u8) {
		if self.value & mask == 0 {
			self.value |= mask;
			self.modified = true;
		}
	}

	fn clr_bit(&mut self, mask: u8) {
		if self.value & mask != 0 {
			self.value &= !mask;
			self.modified = true;
		}
	}

	fn add(&mut self, add: u8, max: u8) {
		if self.value < max {
			self.modified = true;
			if self.value as usize + add as usize > max as usize {
				self.value = max;
			} else {
				self.value += add;
			}
		}
	}

	fn sub(&mut self, sub: u8, min: u8) {
		if self.value > min {
			self.modified = true;
			if self.value as usize > min as usize + sub as usize {
				self.value -= sub;
			} else {
				self.value = min;
			}
		}
	}
}

enum Command<'a> {
	#[allow(dead_code)]
	Joypad(u8),
	Waveform((u8, u8, u8), &'a [u8]),
	Character(u8, u16, u16, (u8, u8, u8), (u8, u8, u8)),
	Rectangle(u16, u16, u16, u16, (u8, u8, u8)),
}

fn read16(bytes: &[u8]) -> u16 {
	assert!(bytes.len() == 2);
	u16::from_le_bytes(bytes.try_into().unwrap())
}

const M8_VENDOR_ID: u16 = 0x16c0;
const M8_PRODUCT_ID: u16 = 0x048a;
const M8_JOYPYAD_CMD: u8 = 0xfb;
const M8_WAVEFORM_CMD: u8 = 0xfc;
const M8_CHARACTER_CMD: u8 = 0xfd;
const M8_RECTANGLE_CMD: u8 = 0xfe;
const M8_SCREEN_WIDTH: u32 = 320;
const M8_SCREEN_HEIGHT: u32 = 240;
const M8_WAVEFORM_HEIGHT: u32 = 22;
const M8_MIN_OCTAVE: u8 = 0;
const M8_MAX_OCTAVE: u8 = 10;
const M8_MIN_VELOCITY: u8 = 0;
const M8_MAX_VELOCITY: u8 = 127;
const M8_KEYS_EDIT: u8 = 1;
const M8_KEYS_OPTION: u8 = 1 << 1;
const M8_KEYS_RIGHT: u8 = 1 << 2;
const M8_KEYS_PLAY: u8 = 1 << 3;
const M8_KEYS_SHIFT: u8 = 1 << 4;
const M8_KEYS_DOWN: u8 = 1 << 5;
const M8_KEYS_UP: u8 = 1 << 6;
const M8_KEYS_LEFT: u8 = 1 << 7;

struct M8 {
	port: Box<dyn SerialPort>,
	buf: [u8; 324],
	slip: Slip<1024>,
	keyjazz: Value<bool>,
	note: Value<u8>,
	octave: Value<u8>,
	velocity: Value<u8>,
	keys: Value<u8>,
}

impl Drop for M8 {
	fn drop(&mut self) {
		let _ = self.disconnect();
	}
}

impl M8 {
	fn open_serial(p: &serialport::SerialPortInfo) -> serialport::Result<Self> {
		if let SerialPortType::UsbPort(ref info) = p.port_type {
			if info.vid == M8_VENDOR_ID && info.pid == M8_PRODUCT_ID {
				return Ok(Self {
					port: serialport::new(&p.port_name, 115200)
						.timeout(Duration::from_millis(10))
						.open()?,
					buf: [0; 324],
					slip: Slip::new(),
					keyjazz: Value::new(false),
					note: Value::new(255),
					octave: Value::new(3),
					velocity: Value::new(100),
					keys: Value::new(0),
				});
			}
		}
		Err(serialport::Error {
			kind: ErrorKind::NoDevice,
			description: "M8 not found".to_string(),
		})
	}

	fn open<T: AsRef<str>>(device: T) -> serialport::Result<Self> {
		for p in available_ports()? {
			if p.port_name != device.as_ref() {
				continue;
			}
			return Self::open_serial(&p);
		}
		Err(serialport::Error {
			kind: ErrorKind::NoDevice,
			description: "Device not found".to_string(),
		})
	}

	fn list_ports() -> serialport::Result<Vec<String>> {
		let mut v = Vec::new();
		for p in available_ports()? {
			if let SerialPortType::UsbPort(ref info) = p.port_type {
				if info.vid == M8_VENDOR_ID && info.pid == M8_PRODUCT_ID {
					v.push(p.port_name.clone())
				}
			}
		}
		Ok(v)
	}

	fn detect() -> serialport::Result<Self> {
		for p in available_ports()? {
			if let Ok(p) = Self::open_serial(&p) {
				return Ok(p);
			}
		}
		Err(serialport::Error {
			kind: ErrorKind::NoDevice,
			description: "No port found".to_string(),
		})
	}

	fn read(&mut self) -> Result<Option<Command<'_>>, String> {
		match self.slip.read(&mut self.port, &mut self.buf) {
			Ok(Some(bytes)) if !bytes.is_empty() => match bytes[0] {
				M8_JOYPYAD_CMD if bytes.len() == 3 => Ok(None),
				M8_JOYPYAD_CMD => Err("invalid joypad command".to_string()),
				M8_WAVEFORM_CMD if bytes.len() == 4 || bytes.len() == 324 => {
					Ok(Some(Command::Waveform((bytes[1], bytes[2], bytes[3]), &bytes[4..])))
				}
				M8_WAVEFORM_CMD => Err("invalid waveform command".to_string()),
				M8_CHARACTER_CMD if bytes.len() == 12 => Ok(Some(Command::Character(
					bytes[1],
					read16(&bytes[2..4]),
					read16(&bytes[4..6]),
					(bytes[6], bytes[7], bytes[8]),
					(bytes[9], bytes[10], bytes[11]),
				))),
				M8_CHARACTER_CMD => Err("invalid character command".to_string()),
				M8_RECTANGLE_CMD if bytes.len() == 12 => Ok(Some(Command::Rectangle(
					read16(&bytes[1..3]),
					read16(&bytes[3..5]),
					read16(&bytes[5..7]),
					read16(&bytes[7..9]),
					(bytes[9], bytes[10], bytes[11]),
				))),
				M8_RECTANGLE_CMD => Err("invalid rectangle command".to_string()),
				_ => Err(format!("unknown command {:02X}", bytes[0])),
			},
			Ok(None) => Ok(None),
			Ok(_) => Err("empty command".to_string()),
			Err(e) => Err(format!("read failed: {:?}", e)),
		}
	}

	fn write(&mut self, buf: &[u8]) -> Result<(), String> {
		match self.port.write(buf) {
			Ok(n) if n != buf.len() => Err("failed to write command".to_string()),
			Ok(_) => Ok(()),
			Err(e) => Err(format!("write failed: {:?}", e)),
		}
	}

	fn enable_and_reset_display(&mut self) -> Result<(), String> {
		self.write("ER".as_bytes())
	}

	fn reset_display(&mut self) -> Result<(), String> {
		self.write("R".as_bytes())
	}

	fn disconnect(&mut self) -> Result<(), String> {
		self.write("D".as_bytes())
	}

	fn reset(&mut self, disconnect: bool) -> Result<(), String> {
		if disconnect {
			let _ = self.disconnect()?;
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
		self.enable_and_reset_display()
	}

	fn send_keyjazz(&mut self) -> Result<(), String> {
		if *self.note == 255 {
			self.write(&[b'K', *self.note])
		} else {
			self.write(&[b'K', *self.note, *self.velocity])
		}
	}

	fn send_keys(&mut self) -> Result<(), String> {
		self.write(&[b'C', *self.keys])
	}

	fn dec_octave(&mut self) {
		self.octave.sub(1, M8_MIN_OCTAVE)
	}

	fn inc_octave(&mut self) {
		self.octave.add(1, M8_MAX_OCTAVE)
	}

	fn dec_velocity(&mut self, fast: bool) {
		self.velocity.sub(if fast { 16 } else { 1 }, M8_MIN_VELOCITY)
	}

	fn inc_velocity(&mut self, fast: bool) {
		self.velocity.add(if fast { 16 } else { 1 }, M8_MAX_VELOCITY)
	}

	fn set_note_off(&mut self) {
		self.note.set(255)
	}

	fn set_note(&mut self, note: u8) {
		self.note.set(note + *self.octave * 12)
	}
}

const FONT_WIDTH: u32 = 128;
const FONT_HEIGHT: u32 = 64;
const FONT_CHARS_BY_ROW: u32 = 16;
const FONT_CHARS_BY_COL: u32 = 8;
const CHAR_WIDTH: u32 = FONT_WIDTH / FONT_CHARS_BY_ROW;
const CHAR_HEIGHT: u32 = FONT_HEIGHT / FONT_CHARS_BY_COL;

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
		let rect = rect::Rect::new(0, 0, M8_SCREEN_WIDTH, M8_WAVEFORM_HEIGHT);
		canvas.fill_rect(rect)?;
		if data.is_empty() {
			return Ok(());
		}
		canvas.set_draw_color(fg);

		let mut points = [rect::Point::new(0, 0); M8_SCREEN_WIDTH as usize];
		for (i, p) in data.iter().enumerate() {
			points[i] = rect::Point::new(i as i32, *p as i32);
		}
		canvas.draw_points(points.as_ref())
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
			self.draw_character(canvas, font, ch as u8, x, y + 3, fg, bg)?;
			x += FONT_WIDTH as u16 / FONT_CHARS_BY_ROW as u16;
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
		let row = c as u32 / FONT_CHARS_BY_ROW;
		let col = c as u32 % FONT_CHARS_BY_ROW;
		let src_rect = rect::Rect::new(
			(col * CHAR_WIDTH) as i32,
			(row * CHAR_HEIGHT) as i32,
			CHAR_WIDTH,
			CHAR_HEIGHT,
		);
		let dst_rect =
			rect::Rect::new(x as i32, y as i32 + 3, src_rect.w as u32, src_rect.h as u32);
		font.set_color_mod(fg.0, fg.1, fg.2);
		if fg != bg {
			let bg_rect =
				rect::Rect::new(x as i32 - 1, y as i32 + 2, CHAR_WIDTH - 1, CHAR_HEIGHT + 1);
			canvas.set_draw_color(bg);
			canvas.fill_rect(bg_rect)?;
		}
		canvas.copy(font, src_rect, dst_rect)
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
		if x == 0 && y == 0 && w == M8_SCREEN_WIDTH as u16 && h == M8_SCREEN_HEIGHT as u16 {
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
		let x = M8_SCREEN_WIDTH - CHAR_WIDTH;
		let y = M8_SCREEN_HEIGHT - CHAR_HEIGHT;

		let rect = rect::Rect::new(x as i32 + 1, y as i32 - 1, CHAR_WIDTH - 1, CHAR_HEIGHT + 1);
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
		let mut x = M8_SCREEN_WIDTH - CHAR_WIDTH * 3 + 2;
		let y = M8_SCREEN_HEIGHT - CHAR_HEIGHT;

		let rect = rect::Rect::new(x as i32 - 1, y as i32 - 1, CHAR_WIDTH * 2 - 2, CHAR_HEIGHT + 1);
		canvas.set_draw_color(self.bg);
		canvas.fill_rect(rect)?;

		if show {
			let (vh, vl) = (velocity >> 4, velocity & 0xf);
			let fg = (0xff, 0xff, 0xff);
			let bg = (0xff, 0, 0);
			let c1 = if vh > 9 { vh - 10 + b'A' } else { vh + b'0' };
			self.draw_character(canvas, font, c1, x as u16, y as u16 - 3, fg, bg)?;
			x += CHAR_WIDTH - 1;
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
		let x = M8_SCREEN_WIDTH as u16 / 2 - s.len() as u16 * CHAR_WIDTH as u16 / 2 + 1;
		self.draw_string(canvas, font, s, x, y, fg, bg)
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
			let y1 = M8_SCREEN_HEIGHT / 3;
			let color1 = (0x32, 0xec, 0xff);
			self.draw_string_centered(canvas, font, "CONFIG", y1 as u16, color1, color1)?;
			let y2 = M8_SCREEN_HEIGHT / 4 * 3;
			let color2 = (0x32, 0xec, 0xff);
			self.draw_string_centered(canvas, font, "ESC = ABORT", y2 as u16, color2, color2)?;
		}

		let color = (0x60, 0x60, 0x8e);
		let y1 = M8_SCREEN_HEIGHT / 2 - CHAR_HEIGHT;
		let y2 = M8_SCREEN_HEIGHT / 2 + CHAR_HEIGHT;
		let rect = rect::Rect::new(0, y1 as i32 + 6, M8_SCREEN_WIDTH, y2 + CHAR_HEIGHT - y1);
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

struct M8Key(Scancode);

impl std::ops::Deref for M8Key {
	type Target = Scancode;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<Scancode> for M8Key {
	fn from(scancode: Scancode) -> Self {
		Self(scancode)
	}
}

impl Serialize for M8Key {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(self.0.name())
	}
}

impl<'de> Deserialize<'de> for M8Key {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		match Scancode::from_name(&s) {
			Some(code) => Ok(Self(code)),
			None => Err(de::Error::custom(&format!("Invalid key: {}", s))),
		}
	}
}

const CONFIG_FILE: &str = "rm8.json";

#[derive(Serialize, Deserialize)]
struct Config {
	up: M8Key,
	down: M8Key,
	left: M8Key,
	right: M8Key,
	shift: M8Key,
	play: M8Key,
	edit: M8Key,
	option: M8Key,
	keyjazz: M8Key,
	octave_plus: M8Key,
	octave_minus: M8Key,
	velocity_plus: M8Key,
	velocity_minus: M8Key,
	fullscreen: bool,
	#[serde(skip)]
	overlap: bool,
}

impl std::default::Default for Config {
	fn default() -> Self {
		Self {
			up: Scancode::Up.into(),
			down: Scancode::Down.into(),
			left: Scancode::Left.into(),
			right: Scancode::Right.into(),
			shift: Scancode::LShift.into(),
			play: Scancode::Space.into(),
			edit: Scancode::LCtrl.into(),
			option: Scancode::LAlt.into(),
			keyjazz: Scancode::Return.into(),
			octave_plus: Scancode::RightBracket.into(),
			octave_minus: Scancode::LeftBracket.into(),
			velocity_plus: Scancode::Equals.into(),
			velocity_minus: Scancode::Minus.into(),
			fullscreen: false,
			overlap: false,
		}
	}
}

const KEYJAZZ: [Scancode; 34] = [
	Scancode::Z,
	Scancode::S,
	Scancode::X,
	Scancode::D,
	Scancode::C,
	Scancode::V,
	Scancode::G,
	Scancode::B,
	Scancode::H,
	Scancode::N,
	Scancode::J,
	Scancode::M,
	Scancode::Comma,
	Scancode::L,
	Scancode::Period,
	Scancode::Semicolon,
	Scancode::Slash,
	Scancode::Q,
	Scancode::Num2,
	Scancode::W,
	Scancode::Num3,
	Scancode::E,
	Scancode::R,
	Scancode::Num5,
	Scancode::T,
	Scancode::Num6,
	Scancode::Y,
	Scancode::Num7,
	Scancode::U,
	Scancode::I,
	Scancode::Num9,
	Scancode::O,
	Scancode::Num0,
	Scancode::P,
];

impl Config {
	fn read<T: AsRef<str>>(&mut self, file: T) -> Result<(), String> {
		let content = std::fs::read_to_string(file.as_ref()).map_err(|e| e.to_string())?;
		let config: Self = serde_json::from_str(&content).map_err(|e| e.to_string())?;
		*self = config;
		self.check_overlap();
		Ok(())
	}

	fn write<T: AsRef<str>>(&self, file: T) -> Result<(), String> {
		let config = self.dump()?;
		std::fs::write(file.as_ref(), config).map_err(|e| e.to_string())
	}

	fn dump(&self) -> Result<String, String> {
		serde_json::to_string_pretty(self).map_err(|e| e.to_string())
	}

	fn check_overlap(&mut self) {
		self.overlap = KEYJAZZ.contains(&*self.up)
			|| KEYJAZZ.contains(&*self.down)
			|| KEYJAZZ.contains(&*self.left)
			|| KEYJAZZ.contains(&*self.right)
			|| KEYJAZZ.contains(&*self.shift)
			|| KEYJAZZ.contains(&*self.play)
			|| KEYJAZZ.contains(&*self.edit)
			|| KEYJAZZ.contains(&*self.option)
			|| KEYJAZZ.contains(&*self.keyjazz)
			|| KEYJAZZ.contains(&*self.octave_minus)
			|| KEYJAZZ.contains(&*self.octave_plus)
			|| KEYJAZZ.contains(&*self.velocity_minus)
			|| KEYJAZZ.contains(&*self.velocity_plus);
	}

	fn handle_keys(&self, m8: &mut M8, code: Scancode, on: bool) {
		let mut mask: u8 = 0;
		if code == *self.up {
			mask |= M8_KEYS_UP;
		} else if code == *self.down {
			mask |= M8_KEYS_DOWN;
		} else if code == *self.left {
			mask |= M8_KEYS_LEFT;
		} else if code == *self.right {
			mask |= M8_KEYS_RIGHT;
		} else if code == *self.shift {
			mask |= M8_KEYS_SHIFT;
		} else if code == *self.play {
			mask |= M8_KEYS_PLAY;
		} else if code == *self.edit {
			mask |= M8_KEYS_EDIT;
		} else if code == *self.option {
			mask |= M8_KEYS_OPTION;
		}
		if on {
			m8.keys.set_bit(mask);
		} else {
			m8.keys.clr_bit(mask);
		}
	}

	fn handle_keyjazz(&mut self, m8: &mut M8, code: Scancode, fast: bool) {
		match code {
			Scancode::Z => m8.set_note(0),
			Scancode::S => m8.set_note(1),
			Scancode::X => m8.set_note(2),
			Scancode::D => m8.set_note(3),
			Scancode::C => m8.set_note(4),
			Scancode::V => m8.set_note(5),
			Scancode::G => m8.set_note(6),
			Scancode::B => m8.set_note(7),
			Scancode::H => m8.set_note(8),
			Scancode::N => m8.set_note(9),
			Scancode::J => m8.set_note(10),
			Scancode::M => m8.set_note(11),
			Scancode::Comma => m8.set_note(12),
			Scancode::L => m8.set_note(13),
			Scancode::Period => m8.set_note(14),
			Scancode::Semicolon => m8.set_note(15),
			Scancode::Slash => m8.set_note(16),
			Scancode::Q => m8.set_note(12),
			Scancode::Num2 => m8.set_note(13),
			Scancode::W => m8.set_note(14),
			Scancode::Num3 => m8.set_note(15),
			Scancode::E => m8.set_note(16),
			Scancode::R => m8.set_note(17),
			Scancode::Num5 => m8.set_note(18),
			Scancode::T => m8.set_note(19),
			Scancode::Num6 => m8.set_note(20),
			Scancode::Y => m8.set_note(21),
			Scancode::Num7 => m8.set_note(22),
			Scancode::U => m8.set_note(23),
			Scancode::I => m8.set_note(24),
			Scancode::Num9 => m8.set_note(25),
			Scancode::O => m8.set_note(26),
			Scancode::Num0 => m8.set_note(27),
			Scancode::P => m8.set_note(28),
			_ => {
				if code == *self.octave_minus {
					m8.dec_octave();
				} else if code == *self.octave_plus {
					m8.inc_octave();
				} else if code == *self.velocity_minus {
					m8.dec_velocity(fast);
				} else if code == *self.velocity_plus {
					m8.inc_velocity(fast);
				}
			}
		}
	}
}

const KEY_NAMES: [&str; 13] = [
	"UP",
	"DOWN",
	"LEFT",
	"RIGHT",
	"SHIFT",
	"PLAY",
	"EDIT",
	"OPTION",
	"KEYJAZZ",
	"OCTAVE+",
	"OCTAVE-",
	"VELOCITY+",
	"VELOCITY-",
];

struct Remap {
	mapping: [Scancode; 13],
	item: usize,
	exists: bool,
	init: bool,
}

impl Remap {
	fn new() -> Self {
		Self { mapping: [Scancode::A; 13], item: 0, exists: false, init: true }
	}

	fn map(&mut self, key: Scancode) {
		self.exists = false;
		if self.done() {
			return;
		}
		for (i, m) in self.mapping.iter().enumerate() {
			if i >= self.item {
				break;
			}
			if *m == key {
				self.exists = true;
				return;
			}
		}
		self.mapping[self.item] = key;
		self.item += 1;
		self.init = false;
	}

	fn done(&self) -> bool {
		self.item >= self.mapping.len()
	}

	fn current(&self) -> &'static str {
		KEY_NAMES[self.item]
	}

	fn write(&self, config: &mut Config) {
		config.up = self.mapping[0].into();
		config.down = self.mapping[1].into();
		config.left = self.mapping[2].into();
		config.right = self.mapping[3].into();
		config.shift = self.mapping[4].into();
		config.play = self.mapping[5].into();
		config.edit = self.mapping[6].into();
		config.option = self.mapping[7].into();
		config.keyjazz = self.mapping[8].into();
		config.octave_plus = self.mapping[9].into();
		config.octave_minus = self.mapping[10].into();
		config.velocity_plus = self.mapping[11].into();
		config.velocity_minus = self.mapping[12].into();
		config.check_overlap();
	}
}

enum Action {
	Continue,
	Return,
}

const USAGE: &str = "Usage rm8 [options]
Available options:
	-help		Display this help screen
	-list		List available M8 devices
	-dev DEVICE	Connect to the the given M8 device
	-wc		Write the default configuration to the standard output
	-wc FILE	Write the default configuration to the given file
	-rc FILE	Read the configuration from the given file";

fn handle_command_line(
	config: &mut Config,
	config_file: &mut Option<String>,
	device: &mut Option<String>,
) -> Result<Action, String> {
	let mut args = std::env::args().skip(1);
	match (args.next().as_deref(), args.next()) {
		(Some("-help"), None) => eprintln!("{}", USAGE),
		(Some("-list"), None) => {
			let ports = M8::list_ports().map_err(|e| e.to_string())?;
			println!("{}", if ports.is_empty() { "No M8 found" } else { "M8 found:" });
			for port in ports {
				println!("\t- {}", port);
			}
		}
		(Some("-wc"), Some(file)) => {
			if let Err(e) = config.write(&file) {
				return Err(format!("Error: writing config to file {} ({})", &file, e));
			}
			config_file.replace(file);
		}
		(Some("-wc"), None) => match config.dump() {
			Ok(json) => println!("{}", json),
			Err(e) => return Err(format!("Error: dumping config ({})", e)),
		},
		(Some("-rc"), Some(file)) => {
			if let Err(e) = config.read(&file) {
				return Err(format!("Error: loading config file `{}` ({})", file, e));
			}
			return Ok(Action::Continue);
		}
		(Some("-rc"), None) => return Err("Error: missing config file argument".to_string()),
		(Some("-dev"), Some(dev)) => {
			device.replace(dev);
			return Ok(Action::Continue);
		}
		(Some("-dev"), None) => return Err("Error: missing device argument".to_string()),
		_ => return Ok(Action::Continue),
	};
	Ok(Action::Return)
}

fn main() -> Result<(), String> {
	let mut config = Config::default();
	let _ = config.read(CONFIG_FILE);

	// process command line arguments
	let mut device: Option<String> = None;
	let mut config_file: Option<String> = None;
	if let Action::Return = handle_command_line(&mut config, &mut config_file, &mut device)? {
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
		.window("rm8", M8_SCREEN_WIDTH * 4, M8_SCREEN_HEIGHT * 4)
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
	canvas.set_logical_size(M8_SCREEN_WIDTH, M8_SCREEN_HEIGHT).map_err(|e| e.to_string())?;

	let creator = canvas.texture_creator();
	// create display texture
	let mut texture = creator
		.create_texture_target(PixelFormatEnum::ARGB8888, M8_SCREEN_WIDTH, M8_SCREEN_HEIGHT)
		.map_err(|e| e.to_string())?;

	// prepare font texture
	let mut font = creator
		.create_texture(PixelFormatEnum::ARGB8888, TextureAccess::Static, FONT_WIDTH, FONT_HEIGHT)
		.map_err(|e| e.to_string())?;

	{
		let mut pixels32 = unsafe {
			*mem::MaybeUninit::<[u32; (FONT_WIDTH * FONT_HEIGHT) as usize]>::uninit().as_mut_ptr()
		};
		for (i, p) in FONT_DATA.iter().enumerate() {
			for j in 0..8 {
				pixels32[i * 8 + j] = if ((*p as usize) & (1 << j)) == 0 { u32::MAX } else { 0 }
			}
		}
		let pixels = unsafe { pixels32.align_to::<u8>().1 };
		font.update(None, pixels, FONT_WIDTH as usize * mem::size_of::<u32>())
			.map_err(|e| e.to_string())?;
	}
	font.set_blend_mode(BlendMode::Blend);

	// key remapper
	let mut remap: Option<Remap> = None;

	let mut event_pump = sdl_context.event_pump().unwrap();
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
									config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
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
					if KEYJAZZ.contains(&scancode) {
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
				while let Some(cmd) = m8.read().unwrap() {
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

const FONT_DATA: &[u8] = &[
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xff, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0,
	0xff, 0xe6, 0xfe, 0xee, 0xe6, 0xfe, 0xee, 0xe6, 0xfe, 0xee, 0xe6, 0xfe, 0xee, 0xe6, 0xfe, 0xee,
	0xff, 0xfe, 0xee, 0xfe, 0xfe, 0xee, 0xfe, 0xfe, 0xee, 0xfe, 0xfe, 0xee, 0xfe, 0xfe, 0xee, 0xfe,
	0xff, 0xef, 0xef, 0xee, 0xef, 0xef, 0xee, 0xef, 0xef, 0xee, 0xef, 0xef, 0xee, 0xef, 0xef, 0xee,
	0xff, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee,
	0xff, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xe0, 0xe0, 0xe2, 0xe0, 0xe0, 0xe2, 0xe0, 0xe0, 0xe2, 0xe0, 0xe0, 0xe2, 0xe0, 0xe0, 0xe2, 0xe0,
	0xfe, 0xfe, 0xee, 0xee, 0xe6, 0xee, 0xfe, 0xfe, 0xee, 0xee, 0xe6, 0xee, 0xfe, 0xfe, 0xee, 0xee,
	0xee, 0xee, 0xee, 0xff, 0xfe, 0xee, 0xee, 0xee, 0xee, 0xff, 0xfe, 0xee, 0xee, 0xee, 0xee, 0xff,
	0xef, 0xef, 0xfe, 0xee, 0xef, 0xee, 0xef, 0xef, 0xfe, 0xee, 0xef, 0xfe, 0xef, 0xef, 0xfe, 0xee,
	0xee, 0xee, 0xee, 0xee, 0xfe, 0xee, 0xee, 0xee, 0xee, 0xee, 0xfe, 0xee, 0xee, 0xee, 0xee, 0xee,
	0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xff, 0xfe, 0xf5, 0xff, 0xfb, 0xff, 0xf9, 0xfb, 0xef, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xef,
	0xff, 0xfe, 0xf5, 0xf5, 0xe0, 0xec, 0xf6, 0xfb, 0xf7, 0xfd, 0xf5, 0xff, 0xff, 0xff, 0xff, 0xef,
	0xff, 0xfe, 0xff, 0xe0, 0xfa, 0xf4, 0xfa, 0xff, 0xf7, 0xfd, 0xfb, 0xfb, 0xff, 0xff, 0xff, 0xf7,
	0xff, 0xfe, 0xff, 0xf5, 0xe0, 0xfb, 0xed, 0xff, 0xf7, 0xfd, 0xf5, 0xf1, 0xff, 0xf1, 0xff, 0xfb,
	0xff, 0xfe, 0xff, 0xe0, 0xeb, 0xe5, 0xea, 0xff, 0xf7, 0xfd, 0xff, 0xfb, 0xff, 0xff, 0xff, 0xfd,
	0xff, 0xff, 0xff, 0xf5, 0xe0, 0xe6, 0xf6, 0xff, 0xf7, 0xfd, 0xff, 0xff, 0xfb, 0xff, 0xff, 0xfe,
	0xff, 0xfe, 0xff, 0xff, 0xfb, 0xff, 0xe9, 0xff, 0xef, 0xfe, 0xff, 0xff, 0xfb, 0xff, 0xfb, 0xfe,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xe0, 0xf8, 0xe0, 0xe0, 0xee, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf1,
	0xee, 0xfb, 0xef, 0xef, 0xee, 0xfe, 0xfe, 0xef, 0xee, 0xee, 0xff, 0xff, 0xf7, 0xff, 0xfd, 0xee,
	0xe6, 0xfb, 0xef, 0xef, 0xee, 0xfe, 0xfe, 0xef, 0xee, 0xee, 0xfd, 0xfd, 0xfb, 0xf1, 0xfb, 0xef,
	0xea, 0xfb, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xf7, 0xe0, 0xe0, 0xff, 0xff, 0xfd, 0xff, 0xf7, 0xf7,
	0xec, 0xfb, 0xfe, 0xef, 0xef, 0xef, 0xee, 0xfb, 0xee, 0xef, 0xff, 0xff, 0xfb, 0xf1, 0xfb, 0xfb,
	0xee, 0xfb, 0xfe, 0xef, 0xef, 0xef, 0xee, 0xfb, 0xee, 0xef, 0xfd, 0xfd, 0xf7, 0xff, 0xfd, 0xff,
	0xe0, 0xe0, 0xe0, 0xe0, 0xef, 0xe0, 0xe0, 0xfb, 0xe0, 0xef, 0xff, 0xfd, 0xff, 0xff, 0xff, 0xfb,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xf1, 0xf1, 0xf0, 0xf1, 0xf0, 0xe0, 0xe0, 0xf1, 0xee, 0xe0, 0xef, 0xee, 0xfe, 0xee, 0xee, 0xf1,
	0xee, 0xee, 0xee, 0xee, 0xee, 0xfe, 0xfe, 0xee, 0xee, 0xfb, 0xef, 0xf6, 0xfe, 0xe4, 0xec, 0xee,
	0xe2, 0xee, 0xee, 0xfe, 0xee, 0xfe, 0xfe, 0xfe, 0xee, 0xfb, 0xef, 0xfa, 0xfe, 0xea, 0xea, 0xee,
	0xea, 0xe0, 0xf0, 0xfe, 0xee, 0xf0, 0xf0, 0xfe, 0xe0, 0xfb, 0xef, 0xfc, 0xfe, 0xee, 0xe6, 0xee,
	0xe2, 0xee, 0xee, 0xfe, 0xee, 0xfe, 0xfe, 0xe6, 0xee, 0xfb, 0xee, 0xfa, 0xfe, 0xee, 0xee, 0xee,
	0xfe, 0xee, 0xee, 0xee, 0xee, 0xfe, 0xfe, 0xee, 0xee, 0xfb, 0xee, 0xf6, 0xfe, 0xee, 0xee, 0xee,
	0xf1, 0xee, 0xf0, 0xf1, 0xf0, 0xe0, 0xfe, 0xf1, 0xee, 0xe0, 0xf1, 0xee, 0xe0, 0xee, 0xee, 0xf1,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xf0, 0xf1, 0xf0, 0xe1, 0xe0, 0xee, 0xee, 0xee, 0xee, 0xee, 0xe0, 0xe7, 0xfe, 0xfc, 0xfb, 0xff,
	0xee, 0xee, 0xee, 0xfe, 0xfb, 0xee, 0xee, 0xee, 0xee, 0xee, 0xef, 0xf7, 0xfe, 0xfd, 0xf5, 0xff,
	0xee, 0xee, 0xee, 0xfe, 0xfb, 0xee, 0xee, 0xee, 0xf5, 0xee, 0xf7, 0xf7, 0xfd, 0xfd, 0xff, 0xff,
	0xf0, 0xee, 0xf0, 0xf1, 0xfb, 0xee, 0xee, 0xee, 0xfb, 0xe1, 0xfb, 0xf7, 0xfb, 0xfd, 0xff, 0xff,
	0xfe, 0xea, 0xf6, 0xef, 0xfb, 0xee, 0xf5, 0xea, 0xf5, 0xef, 0xfd, 0xf7, 0xf7, 0xfd, 0xff, 0xff,
	0xfe, 0xf6, 0xee, 0xef, 0xfb, 0xee, 0xf5, 0xe4, 0xee, 0xef, 0xfe, 0xf7, 0xef, 0xfd, 0xff, 0xff,
	0xfe, 0xe9, 0xee, 0xf0, 0xfb, 0xf1, 0xfb, 0xee, 0xee, 0xf0, 0xe0, 0xe7, 0xef, 0xfc, 0xff, 0xe0,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfb, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xfe, 0xff, 0xfe, 0xff, 0xef, 0xff, 0xe3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xff, 0xe1, 0xf0, 0xe1, 0xe1, 0xf1, 0xfd, 0xe0, 0xfe, 0xfb, 0xef, 0xee, 0xfe, 0xe4, 0xf0, 0xf1,
	0xff, 0xee, 0xee, 0xfe, 0xee, 0xee, 0xfd, 0xee, 0xf0, 0xfb, 0xef, 0xf6, 0xfe, 0xea, 0xee, 0xee,
	0xff, 0xee, 0xee, 0xfe, 0xee, 0xe0, 0xe0, 0xe0, 0xee, 0xfb, 0xef, 0xf8, 0xfe, 0xea, 0xee, 0xee,
	0xff, 0xee, 0xee, 0xfe, 0xee, 0xfe, 0xfd, 0xef, 0xee, 0xfb, 0xef, 0xf6, 0xfe, 0xea, 0xee, 0xee,
	0xff, 0xe1, 0xf0, 0xe1, 0xe1, 0xe1, 0xfd, 0xf1, 0xee, 0xf1, 0xf0, 0xee, 0xe1, 0xea, 0xee, 0xf1,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xef, 0xfe, 0xfe, 0xff, 0xff,
	0xff, 0xff, 0xff, 0xff, 0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf7, 0xfe, 0xfd, 0xed, 0xff,
	0xe0, 0xe1, 0xe1, 0xe1, 0xe0, 0xee, 0xee, 0xea, 0xee, 0xee, 0xe0, 0xf7, 0xfe, 0xfd, 0xf2, 0xff,
	0xee, 0xee, 0xfe, 0xfe, 0xfd, 0xee, 0xee, 0xea, 0xf5, 0xee, 0xf7, 0xef, 0xfe, 0xfe, 0xff, 0xff,
	0xee, 0xee, 0xfe, 0xe0, 0xfd, 0xee, 0xee, 0xea, 0xfb, 0xe1, 0xfb, 0xf7, 0xfe, 0xfd, 0xff, 0xff,
	0xf0, 0xe0, 0xfe, 0xef, 0xfd, 0xee, 0xf5, 0xea, 0xf5, 0xef, 0xfd, 0xf7, 0xfe, 0xfd, 0xff, 0xff,
	0xfe, 0xef, 0xfe, 0xf0, 0xe3, 0xf1, 0xfb, 0xf5, 0xee, 0xf0, 0xe0, 0xef, 0xfe, 0xfe, 0xff, 0xff,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];
