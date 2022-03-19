use sdl2::keyboard::Keycode;
use std::fmt::Write;

use crate::{
	config::{self, Command},
	draw::{Context, LINE_HEIGHT},
	font,
	m8::M8,
};

fn inc_hex(hex: &mut u8, add: u8) -> bool {
	if *hex < 0xff - add {
		*hex += add;
		return true;
	} else if *hex != 0xff {
		*hex = 0xff;
		return true;
	}
	false
}

fn dec_hex(hex: &mut u8, sub: u8) -> bool {
	if *hex > sub {
		*hex -= sub;
		return true;
	} else if *hex != 0 {
		*hex = 0;
		return true;
	}
	false
}

pub enum Direction {
	Above,
	Below,
	Left,
	Right,
}

#[derive(Debug)]
pub enum Edit {
	Next(bool),
	Prev(bool),
	Click,
	Reset,
}

#[derive(Debug)]
pub struct Bool {
	init: bool,
	value: bool,
}

impl Bool {
	pub fn new(value: bool) -> Self {
		Self { init: value, value }
	}

	pub fn value(&self) -> bool {
		self.value
	}

	pub fn set_value(&mut self, value: bool) {
		self.value = value;
	}
}

#[derive(Debug)]
pub struct Int {
	init: usize,
	value: usize,
	min: usize,
	max: usize,
	step: usize,
}

impl Int {
	pub fn new(value: usize, min: usize, max: usize, step: usize) -> Self {
		Self { init: value, value, max, min, step }
	}

	pub fn value(&self) -> usize {
		self.value
	}

	pub fn set_max(&mut self, max: usize) {
		if max >= self.min {
			self.max = max;
			if self.value > max {
				self.value = max;
			}
		}
	}

	pub fn set_value(&mut self, value: usize) {
		if value >= self.min && value <= self.max {
			self.value = value;
		}
	}
}

#[derive(Debug)]
pub struct Key {
	init: Keycode,
	value: Keycode,
	selected: bool,
	exists: bool,
}

impl Key {
	pub fn new(value: Keycode) -> Self {
		Self { init: value, value, selected: false, exists: false }
	}

	pub fn value(&self) -> Keycode {
		self.value
	}

	pub fn set_value(&mut self, value: Keycode) {
		self.value = value;
	}

	pub fn focus(&mut self) {
		self.selected = true;
	}

	pub fn exists(&mut self) {
		self.exists = true;
	}

	pub fn unfocus(&mut self) {
		self.selected = false;
		self.exists = false;
	}
}

#[derive(Debug)]
pub struct Font {
	init: config::Font,
	value: config::Font,
}

impl Font {
	pub fn new(value: config::Font) -> Self {
		Self { init: value, value }
	}

	pub fn value(&self) -> config::Font {
		self.value
	}

	pub fn set_value(&mut self, value: config::Font) {
		self.value = value;
	}
}

#[derive(Debug)]
pub struct Rgb {
	r: u8,
	g: u8,
	b: u8,
	r_init: u8,
	g_init: u8,
	b_init: u8,
}

impl Rgb {
	pub fn new(rgb: config::Rgb) -> Self {
		Self { r: rgb.0, g: rgb.1, b: rgb.2, r_init: rgb.0, g_init: rgb.1, b_init: rgb.2 }
	}

	pub fn value(&self) -> config::Rgb {
		config::Rgb(self.r, self.g, self.b)
	}

	pub fn set_value(&mut self, rgb: config::Rgb) {
		self.r = rgb.0;
		self.g = rgb.1;
		self.b = rgb.2;
	}
}

#[derive(Debug)]
pub struct Device {
	list: Vec<String>,
	selected: usize,
}

impl Device {
	pub fn new(device: Option<String>) -> Self {
		let list = M8::list_ports().unwrap_or_default();
		let selected =
			device.and_then(|ref dev| list.iter().position(|name| dev == name)).unwrap_or(0);
		Self { list, selected }
	}

	pub fn value(&self) -> Option<&str> {
		if self.selected < self.list.len() {
			return Some(&self.list[self.selected]);
		}
		None
	}

	fn update_list(&mut self) {
		if let Ok(list) = M8::list_ports() {
			if list != self.list {
				self.list = list;
				if self.selected >= self.list.len() {
					self.selected = self.list.len() - 1;
				}
			}
		}
	}
}

#[derive(Debug)]
pub struct Audio {
	list: Vec<String>,
	selected: usize,
}

impl Audio {
	pub fn new(device: Option<String>) -> Self {
		let list = M8::list_capture_devices().unwrap_or_default();
		let selected =
			device.and_then(|ref dev| list.iter().position(|name| dev == name)).unwrap_or(0);
		Self { list, selected }
	}

	pub fn value(&self) -> Option<&str> {
		if self.selected < self.list.len() {
			return Some(&self.list[self.selected]);
		}
		None
	}

	fn update_list(&mut self) {
		if let Ok(list) = M8::list_capture_devices() {
			if list != self.list {
				self.list = list;
				if self.selected >= self.list.len() {
					self.selected = self.list.len() - 1;
				}
			}
		}
	}
}

#[derive(Debug)]
pub enum Input {
	Bool(Bool),
	Int(Int),
	Command(Command),
	Command2(Command, Command),
	CommandLabel2(Command, String, usize, Command),
	Key(Key),
	Rgb(Rgb),
	Font(Font),
	Device(Device),
	Audio(Audio),
}

#[derive(Debug, PartialEq)]
pub enum Action {
	None,
	Modified,
	Do(&'static str),
}

impl Action {
	pub fn map(&mut self, other: Action) {
		if self == &Action::None && other != Action::None {
			*self = other;
		}
	}

	pub fn reset(&mut self) {
		*self = Action::None;
	}
}

#[derive(Debug)]
pub enum Item {
	Empty,
	Text(String),
	Label(String, bool),
	Label2(String, String),
	Title2(String, String, usize),
	Input(String, Input),
	Action2(&'static str, &'static str),
	Action3(&'static str, &'static str, &'static str),
}

impl Item {
	pub fn cursors(&self) -> usize {
		match self {
			Item::Action2(..) => 2,
			Item::Action3(..) => 3,
			Item::Input(_, input) => match input {
				Input::Bool(_) | Input::Int(_) | Input::Font(_) | Input::Command(_) => 1,
				Input::Command2(..)
				| Input::CommandLabel2(..)
				| Input::Device(..)
				| Input::Audio(..) => 2,
				Input::Rgb(_) => 3,
				Input::Key(Key { selected, .. }) => {
					if *selected {
						1
					} else {
						0
					}
				}
			},
			Item::Empty | Item::Text(_) | Item::Label(..) | Item::Label2(..) | Item::Title2(..) => {
				0
			}
		}
	}

	pub fn edit(&mut self, edit: Edit, cursor: usize) -> Action {
		match self {
			Item::Empty | Item::Text(_) | Item::Label(..) | Item::Label2(..) | Item::Title2(..) => {
			}
			Item::Action2(a1, a2) => {
				if cursor == 0 {
					match edit {
						Edit::Next(_) => {}
						Edit::Prev(_) => {}
						Edit::Reset => {}
						Edit::Click => {
							return Action::Do(a1);
						}
					}
				} else if cursor == 1 {
					match edit {
						Edit::Next(_) => {}
						Edit::Prev(_) => {}
						Edit::Reset => {}
						Edit::Click => {
							return Action::Do(a2);
						}
					}
				}
			}
			Item::Action3(a1, a2, a3) => {
				if cursor == 0 {
					match edit {
						Edit::Next(_) => {}
						Edit::Prev(_) => {}
						Edit::Reset => {}
						Edit::Click => {
							return Action::Do(a1);
						}
					}
				} else if cursor == 1 {
					match edit {
						Edit::Next(_) => {}
						Edit::Prev(_) => {}
						Edit::Reset => {}
						Edit::Click => {
							return Action::Do(a2);
						}
					}
				} else if cursor == 2 {
					match edit {
						Edit::Next(_) => {}
						Edit::Prev(_) => {}
						Edit::Reset => {}
						Edit::Click => {
							return Action::Do(a3);
						}
					}
				}
			}
			Item::Input(_, ref mut input) => match input {
				Input::Bool(b) => match edit {
					Edit::Next(_) | Edit::Prev(_) => {
						b.value = !b.value;
						return Action::Modified;
					}
					Edit::Reset => {
						if b.value != b.init {
							b.value = b.init;
							return Action::Modified;
						}
					}
					Edit::Click => {}
				},
				Input::Int(i) => match edit {
					Edit::Next(fast) => {
						let step = if fast { i.step } else { 1 };
						if i.value + step < i.max {
							i.value += step;
							return Action::Modified;
						} else if i.value != i.max {
							i.value = i.max;
							return Action::Modified;
						}
					}
					Edit::Prev(fast) => {
						let step = if fast { i.step } else { 1 };
						if i.value > i.min + step {
							i.value -= step;
							return Action::Modified;
						} else if i.value != i.min {
							i.value = i.min;
							return Action::Modified;
						}
					}
					Edit::Reset => {
						if i.value != i.init {
							i.value = i.init;
							return Action::Modified;
						}
					}
					Edit::Click => {}
				},
				Input::Command(c) => match edit {
					Edit::Next(_) => {
						if let Ok(cmd) = Command::try_from(*c as u8 + 1) {
							*c = cmd;
							return Action::Modified;
						}
					}
					Edit::Prev(_) => {
						let n = *c as u8;
						if n > 0 {
							if let Ok(cmd) = Command::try_from(n - 1) {
								*c = cmd;
								return Action::Modified;
							}
						}
					}
					Edit::Reset => {
						if c != &Command::None {
							*c = Command::None;
							return Action::Modified;
						}
					}
					Edit::Click => {}
				},
				Input::Command2(c1, c2) | Input::CommandLabel2(c1, _, _, c2) => match edit {
					Edit::Next(_) => {
						if cursor == 0 {
							if let Ok(cmd) = Command::try_from(*c1 as u8 + 1) {
								*c1 = cmd;
								return Action::Modified;
							}
						} else if cursor == 1 {
							if let Ok(cmd) = Command::try_from(*c2 as u8 + 1) {
								*c2 = cmd;
								return Action::Modified;
							}
						}
					}
					Edit::Prev(_) => {
						if cursor == 0 {
							let n = *c1 as u8;
							if n > 0 {
								if let Ok(cmd) = Command::try_from(n - 1) {
									*c1 = cmd;
									return Action::Modified;
								}
							}
						} else if cursor == 1 {
							let n = *c2 as u8;
							if n > 0 {
								if let Ok(cmd) = Command::try_from(n - 1) {
									*c2 = cmd;
									return Action::Modified;
								}
							}
						}
					}
					Edit::Reset => {
						if cursor == 0 {
							*c1 = Command::None;
							return Action::Modified;
						} else if cursor == 1 {
							*c2 = Command::None;
							return Action::Modified;
						}
					}
					Edit::Click => {}
				},
				Input::Key(k) => match edit {
					Edit::Next(_) => {}
					Edit::Prev(_) => {}
					Edit::Reset => {
						if k.value != k.init {
							k.value = k.init;
							return Action::Modified;
						}
					}
					Edit::Click => {
						k.selected = true;
					}
				},
				Input::Rgb(c) => {
					if cursor == 0 {
						match edit {
							Edit::Next(fast) => {
								if inc_hex(&mut c.r, if fast { 16 } else { 1 }) {
									return Action::Modified;
								}
							}
							Edit::Prev(fast) => {
								if dec_hex(&mut c.r, if fast { 16 } else { 1 }) {
									return Action::Modified;
								}
							}
							Edit::Reset => {
								if c.r != c.r_init {
									c.r = c.r_init;
									return Action::Modified;
								}
							}
							Edit::Click => {}
						}
					} else if cursor == 1 {
						match edit {
							Edit::Next(fast) => {
								if inc_hex(&mut c.g, if fast { 16 } else { 1 }) {
									return Action::Modified;
								}
							}
							Edit::Prev(fast) => {
								if dec_hex(&mut c.g, if fast { 16 } else { 1 }) {
									return Action::Modified;
								}
							}
							Edit::Reset => {
								if c.g != c.g_init {
									c.g = c.g_init;
									return Action::Modified;
								}
							}
							Edit::Click => {}
						}
					} else if cursor == 2 {
						match edit {
							Edit::Next(fast) => {
								if inc_hex(&mut c.b, if fast { 16 } else { 1 }) {
									return Action::Modified;
								}
							}
							Edit::Prev(fast) => {
								if dec_hex(&mut c.b, if fast { 16 } else { 1 }) {
									return Action::Modified;
								}
							}
							Edit::Reset => {
								if c.b != c.b_init {
									c.b = c.b_init;
									return Action::Modified;
								}
							}
							Edit::Click => {}
						}
					}
				}
				Input::Font(f) => match edit {
					Edit::Next(_) => {
						if let Ok(font) = config::Font::try_from(f.value as u8 + 1) {
							f.value = font;
							return Action::Modified;
						}
					}
					Edit::Prev(_) => {
						let n = f.value as u8;
						if n > 0 {
							if let Ok(font) = config::Font::try_from(n - 1) {
								f.value = font;
								return Action::Modified;
							}
						}
					}
					Edit::Reset => {
						if f.value != f.init {
							f.value = f.init;
							return Action::Modified;
						}
					}
					Edit::Click => {}
				},
				Input::Device(d) => match edit {
					Edit::Next(_) => {
						if cursor == 0 && !d.list.is_empty() && d.selected + 1 < d.list.len() {
							d.selected += 1;
							return Action::Modified;
						}
					}
					Edit::Prev(_) => {
						if cursor == 0 && !d.list.is_empty() && d.selected > 0 {
							d.selected -= 1;
							return Action::Modified;
						}
					}
					Edit::Reset => {
						if cursor == 0 && d.selected != 0 {
							d.selected = 0;
							return Action::Modified;
						}
					}
					Edit::Click => {
						if cursor == 1 {
							d.update_list();
							return Action::Modified;
						}
					}
				},
				Input::Audio(d) => match edit {
					Edit::Next(_) => {
						if cursor == 0 && !d.list.is_empty() && d.selected + 1 < d.list.len() {
							d.selected += 1;
							return Action::Modified;
						}
					}
					Edit::Prev(_) => {
						if cursor == 0 && !d.list.is_empty() && d.selected > 0 {
							d.selected -= 1;
							return Action::Modified;
						}
					}
					Edit::Reset => {
						if cursor == 0 && d.selected != 0 {
							d.selected = 0;
							return Action::Modified;
						}
					}
					Edit::Click => {
						if cursor == 1 {
							d.update_list();
							return Action::Modified;
						}
					}
				},
			},
		}
		Action::None
	}

	pub fn cursor_rect(&self, cursor: usize) -> (i32, i32, u32, u32) {
		match self {
			Item::Empty | Item::Text(_) | Item::Label(..) | Item::Label2(..) | Item::Title2(..) => {
				(0, 0, 0, 0)
			}
			Item::Action2(a1, a2) => {
				let width = font::width(a1.len());
				if cursor == 0 {
					(0, 0, width as u32, LINE_HEIGHT as u32)
				} else if cursor == 1 {
					(width, 0, font::width(a2.len()) as u32, LINE_HEIGHT as u32)
				} else {
					(0, 0, 0, 0)
				}
			}
			Item::Action3(a1, a2, a3) => {
				let width = font::width(a1.len());
				if cursor == 0 {
					(0, 0, width as u32, LINE_HEIGHT as u32)
				} else if cursor == 1 {
					(width, 0, font::width(a2.len()) as u32, LINE_HEIGHT as u32)
				} else if cursor == 2 {
					let width = width + font::width(a2.len());
					(width, 0, font::width(a3.len()) as u32, LINE_HEIGHT as u32)
				} else {
					(0, 0, 0, 0)
				}
			}
			Item::Input(_, input) => match input {
				Input::Bool(_) => (0, 0, font::width(3) as u32, LINE_HEIGHT as u32),
				Input::Int(i) => {
					let value = if i.value == 0 { 1 } else { i.value };
					let width = font::width(((value as f64).log10() + 1.0).floor() as usize);
					(0, 0, width as u32, LINE_HEIGHT as u32)
				}
				Input::Command(..) => {
					let width = font::width(Command::MAX_LENGTH);
					if cursor == 0 {
						(0, 0, width as u32, LINE_HEIGHT as u32)
					} else {
						(0, 0, 0, 0)
					}
				}
				Input::Command2(..) => {
					let width = font::width(Command::MAX_LENGTH);
					if cursor == 0 {
						(0, 0, width as u32, LINE_HEIGHT as u32)
					} else if cursor == 1 {
						(width, 0, width as u32, LINE_HEIGHT as u32)
					} else {
						(0, 0, 0, 0)
					}
				}
				Input::CommandLabel2(_, _, pad, _) => {
					let width = font::width(Command::MAX_LENGTH);
					if cursor == 0 {
						(0, 0, width as u32, LINE_HEIGHT as u32)
					} else if cursor == 1 {
						let pad = font::width(*pad);
						(width + pad, 0, width as u32, LINE_HEIGHT as u32)
					} else {
						(0, 0, 0, 0)
					}
				}
				Input::Key(k) if k.selected => (0, 0, font::width(12) as u32, LINE_HEIGHT as u32),
				Input::Key(_) => (0, 0, 0, LINE_HEIGHT as u32),
				Input::Rgb(_) => {
					let width = font::width(2);
					if cursor == 0 {
						(0, 0, width as u32, LINE_HEIGHT as u32)
					} else if cursor == 1 {
						(width, 0, width as u32, LINE_HEIGHT as u32)
					} else if cursor == 2 {
						(font::width(5), 0, width as u32, LINE_HEIGHT as u32)
					} else {
						(0, 0, 0, 0)
					}
				}
				Input::Font(_) => {
					(0, 0, font::width(config::Font::MAX_LENGTH) as u32, LINE_HEIGHT as u32)
				}
				Input::Device(_) | Input::Audio(_) => {
					let width = font::width(19);
					if cursor == 0 {
						(0, 0, width as u32, LINE_HEIGHT as u32)
					} else if cursor == 1 {
						(width, 0, font::width(7) as u32, LINE_HEIGHT as u32)
					} else {
						(0, 0, 0, 0)
					}
				}
			},
		}
	}

	pub fn draw(
		&mut self,
		ctx: &mut Context<'_, '_, '_>,
		x: i32,
		y: i32,
		pad: i32,
		cursor: Option<usize>,
	) -> Result<(), String> {
		let fg_default = ctx.theme.text_default;
		let fg_value = ctx.theme.text_value;
		let fg_screen = ctx.theme.screen;
		let fg_title = ctx.theme.text_title;
		let fg_info = ctx.theme.text_info;
		match self {
			Item::Label(label, false) | Item::Label2(label, _) | Item::Input(label, _) => {
				ctx.draw_str(label, x, y, fg_default, fg_default)?
			}
			Item::Empty | Item::Action2(..) | Item::Action3(..) | Item::Title2(..) => {}
			Item::Label(label, true) => ctx.draw_str(label, x, y, fg_title, fg_title)?,
			Item::Text(label) => ctx.draw_str(label, x, y, fg_info, fg_info)?,
		}

		let x = x + pad;
		match self {
			Item::Empty | Item::Text(_) | Item::Label(..) => {}
			Item::Label2(_, ref label) => ctx.draw_str(label, x, y, fg_default, fg_default)?,
			Item::Title2(ref label1, ref label2, size) => {
				ctx.draw_str(label1, x, y, fg_title, fg_title)?;
				ctx.draw_str(label2, x + font::width(*size), y, fg_title, fg_title)?;
			}
			Item::Action2(a1, a2) => {
				let (fg1, fg2) = match cursor {
					Some(0) => (fg_screen, fg_value),
					Some(1) => (fg_value, fg_screen),
					_ => (fg_value, fg_value),
				};
				ctx.draw_str(a1, x, y, fg1, fg1)?;
				ctx.draw_str(a2, x + font::width(a1.len()), y, fg2, fg2)?;
			}
			Item::Action3(a1, a2, a3) => {
				let (fg1, fg2, fg3) = match cursor {
					Some(0) => (fg_screen, fg_value, fg_value),
					Some(1) => (fg_value, fg_screen, fg_value),
					Some(2) => (fg_value, fg_value, fg_screen),
					_ => (fg_value, fg_value, fg_value),
				};
				let mut width = font::width(a1.len());
				ctx.draw_str(a1, x, y, fg1, fg1)?;
				ctx.draw_str(a2, x + width, y, fg2, fg2)?;
				width += font::width(a2.len());
				ctx.draw_str(a3, x + width, y, fg3, fg3)?;
			}
			Item::Input(_, input) => match input {
				Input::Bool(b) => {
					let fg = if cursor.is_some() { fg_screen } else { fg_value };
					ctx.draw_str(if b.value { "yes" } else { "no" }, x, y, fg, fg)?;
				}
				Input::Int(i) => {
					let fg = if cursor.is_some() { fg_screen } else { fg_value };
					let s = format!("{}", i.value);
					ctx.draw_str(&s, x, y, fg, fg)?;
				}
				Input::Command(c) => {
					let fg = match cursor {
						Some(0) => fg_screen,
						_ => fg_value,
					};
					let s = format!("{}", c);
					ctx.draw_str(&s, x, y, fg, fg)?;
				}
				Input::Command2(c1, c2) => {
					let (fg1, fg2) = match cursor {
						Some(0) => (fg_screen, fg_value),
						Some(1) => (fg_value, fg_screen),
						_ => (fg_value, fg_value),
					};
					let mut s = format!("{}", c1);
					ctx.draw_str(&s, x, y, fg1, fg1)?;
					s.clear();
					let _ = write!(&mut s, "{}", c2);
					let width = font::width(Command::MAX_LENGTH);
					ctx.draw_str(&s, x + width, y, fg2, fg2)?;
				}
				Input::CommandLabel2(c1, label, pad, c2) => {
					let (fg1, fg2) = match cursor {
						Some(0) => (fg_screen, fg_value),
						Some(1) => (fg_value, fg_screen),
						_ => (fg_value, fg_value),
					};
					let mut s = format!("{}", c1);
					ctx.draw_str(&s, x, y, fg1, fg1)?;
					s.clear();
					let width = font::width(Command::MAX_LENGTH);
					ctx.draw_str(label, x + width, y, fg_default, fg_default)?;
					let _ = write!(&mut s, "{}", c2);
					ctx.draw_str(&s, x + width + font::width(*pad), y, fg2, fg2)?;
				}
				Input::Key(k) => {
					let fg = if cursor.is_some() { fg_screen } else { fg_value };
					if k.selected {
						if k.exists {
							ctx.draw_str("KEY IS TAKEN", x, y, fg_info, fg_info)?;
						} else {
							ctx.draw_str("PRESS KEY", x, y, fg, fg)?;
						}
					} else {
						let s = format!("{}", k.value);
						ctx.draw_str(&s, x, y, fg, fg)?;
					}
				}
				Input::Rgb(c) => {
					let (fg1, fg2, fg3) = match cursor {
						Some(0) => (fg_screen, fg_value, fg_value),
						Some(1) => (fg_value, fg_screen, fg_value),
						Some(2) => (fg_value, fg_value, fg_screen),
						_ => (fg_value, fg_value, fg_value),
					};
					let width = font::width(2);
					let mut s = format!("{:02X}", c.r);
					ctx.draw_str(&s, x, y, fg1, fg1)?;
					s.clear();
					let _ = write!(&mut s, "{:02X}", c.g);
					ctx.draw_str(&s, x + width, y, fg2, fg2)?;
					s.clear();
					let _ = write!(&mut s, "{:02X}", c.b);
					ctx.draw_str(&s, x + width * 2, y, fg3, fg3)?;
					ctx.draw_rect(
						(x + font::width(8), y, width as u32, LINE_HEIGHT as u32),
						config::Rgb(c.r, c.g, c.b),
					)?;
				}
				Input::Font(f) => {
					let fg = if cursor.is_some() { fg_screen } else { fg_value };
					let s = format!("{}", f.value);
					ctx.draw_str(&s, x, y, fg, fg)?;
				}
				Input::Device(d) => {
					let width = font::width(19);
					let (fg1, fg2) = match cursor {
						Some(0) => (fg_screen, fg_value),
						Some(1) => (fg_value, fg_screen),
						_ => (fg_value, fg_value),
					};
					let dev = d.value().unwrap_or("").trim_start_matches("/dev/");
					ctx.draw_str(dev, x, y, fg1, fg1)?;
					ctx.draw_str("REFRESH", x + width, y, fg2, fg2)?;
				}
				Input::Audio(d) => {
					let width = font::width(19);
					let (fg1, fg2) = match cursor {
						Some(0) => (fg_screen, fg_value),
						Some(1) => (fg_value, fg_screen),
						_ => (fg_value, fg_value),
					};
					let dev = d.value().unwrap_or("");
					ctx.draw_str(dev, x, y, fg1, fg1)?;
					ctx.draw_str("REFRESH", x + width, y, fg2, fg2)?;
				}
			},
		}
		Ok(())
	}
}
