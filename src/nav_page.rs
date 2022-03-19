use sdl2::keyboard::Keycode;
use std::cmp::Ordering;

use crate::{
	config::{self, Command},
	draw::{Context, LINE_HEIGHT},
	font,
	nav_item::{Action, Audio, Bool, Device, Direction, Edit, Font, Input, Int, Item, Key, Rgb},
};

const PAD_X: i32 = 10;
const PAD_Y: i32 = 50;
const TITLE_PAD_Y: i32 = 30;

#[derive(Debug)]
pub struct Page {
	name: String,
	short: char,
	items: Vec<Item>,
	pad: usize,
	cursor: (usize, usize),
	cursor_rect: (i32, i32, u32, u32),
	above: Vec<Page>,
	below: Vec<Page>,
}

impl Page {
	pub fn new<I: Into<String>>(name: I, short: char) -> Self {
		Self {
			name: name.into(),
			short,
			items: vec![],
			pad: 0,
			cursor: (0, 0),
			cursor_rect: (0, 0, 0, 0),
			above: vec![],
			below: vec![],
		}
	}

	pub fn items(&self) -> impl Iterator<Item = &Item> {
		self.items.iter()
	}

	pub fn items_mut(&mut self) -> impl Iterator<Item = &mut Item> {
		self.items.iter_mut()
	}

	pub fn add_page_above(&mut self, mut page: Page) {
		page.reset_cursor();
		self.above.push(page);
	}

	pub fn above(&self) -> &[Page] {
		&self.above
	}

	pub fn below(&self) -> &[Page] {
		&self.below
	}

	pub fn find(&self, short: char) -> Option<&Page> {
		for sub in self.above.iter() {
			if sub.short == short {
				return Some(sub);
			}
		}
		for sub in self.below.iter() {
			if sub.short == short {
				return Some(sub);
			}
		}
		None
	}

	pub fn find_mut(&mut self, short: char) -> Option<&mut Page> {
		for sub in self.above.iter_mut() {
			if sub.short == short {
				return Some(sub);
			}
		}
		for sub in self.below.iter_mut() {
			if sub.short == short {
				return Some(sub);
			}
		}
		None
	}

	pub fn get_mut(&mut self, index: isize) -> Option<&mut Page> {
		match index.cmp(&0) {
			Ordering::Less => self.below.get_mut(index.saturating_abs() as usize - 1),
			Ordering::Equal => Some(self),
			Ordering::Greater => self.above.get_mut(index.saturating_abs() as usize - 1),
		}
	}

	pub fn get(&self, index: isize) -> Option<&Page> {
		match index.cmp(&0) {
			Ordering::Less => self.below.get(index.saturating_abs() as usize - 1),
			Ordering::Equal => Some(self),
			Ordering::Greater => self.above.get(index.saturating_abs() as usize - 1),
		}
	}

	pub fn add_page_below(&mut self, mut page: Page) {
		page.reset_cursor();
		self.below.push(page);
	}

	fn item(&self) -> &Item {
		&self.items[self.cursor.1]
	}

	fn item_mut(&mut self) -> &mut Item {
		&mut self.items[self.cursor.1]
	}

	pub fn reset_cursor(&mut self) {
		for (i, item) in self.items.iter().enumerate() {
			if item.cursors() > 0 {
				self.cursor = (0, i);
				break;
			}
		}
	}

	fn reset_cursor_x(&mut self) {
		let cursors = self.item().cursors();
		if cursors > 0 && self.cursor.0 > cursors - 1 {
			self.cursor.0 = cursors - 1;
		}
	}

	pub fn edit_item(&mut self, edit: Edit) -> Action {
		let cursor = self.cursor.0;
		self.item_mut().edit(edit, cursor)
	}

	pub fn cursor_move(&mut self, direction: Direction) {
		match direction {
			Direction::Above => {
				if self.cursor.1 > 0 {
					let mut cursor = self.cursor.1 - 1;
					loop {
						if self.items[cursor].cursors() > 0 {
							self.cursor.1 = cursor;
							self.reset_cursor_x();
							break;
						}
						if cursor == 0 {
							break;
						}
						cursor -= 1;
					}
				}
			}
			Direction::Below => {
				let max = self.items.len();
				if self.cursor.1 < max {
					let mut cursor = self.cursor.1 + 1;
					while cursor < max {
						if self.items[cursor].cursors() > 0 {
							self.cursor.1 = cursor;
							self.reset_cursor_x();
							break;
						}
						cursor += 1;
					}
				}
			}
			Direction::Left => {
				if self.cursor.0 > 0 {
					self.cursor.0 -= 1;
				}
			}
			Direction::Right => {
				if self.cursor.0 + 1 < self.item().cursors() {
					self.cursor.0 += 1;
				}
			}
		}
	}

	pub fn short_name(&self) -> char {
		self.short
	}

	pub fn draw(&mut self, ctx: &mut Context<'_, '_, '_>) -> Result<(), String> {
		let fg = ctx.theme.text_title;
		ctx.draw_str(&self.name, PAD_X, TITLE_PAD_Y, fg, fg)?;
		let x = PAD_X;
		let mut y = PAD_Y;
		let pad = font::width(self.pad);
		for (i, item) in self.items.iter_mut().enumerate() {
			let cursor = if self.cursor.1 == i && item.cursors() > 0 {
				// clear last cursor
				ctx.draw_rect(self.cursor_rect, ctx.theme.screen)?;
				self.cursor_rect = item.cursor_rect(self.cursor.0);
				self.cursor_rect.0 += x + pad;
				if self.cursor_rect.0 >= 2 {
					self.cursor_rect.0 -= 2;
				}
				self.cursor_rect.1 += y + 1;
				if self.cursor_rect.2 > 0 {
					self.cursor_rect.2 -= 1;
				}
				self.cursor_rect.3 += 1;
				// draw cursor
				ctx.draw_rect(self.cursor_rect, ctx.theme.cursor)?;
				Some(self.cursor.0)
			} else {
				None
			};
			item.draw(ctx, x, y, pad, cursor)?;
			y += LINE_HEIGHT;
		}
		Ok(())
	}

	pub fn add_item(&mut self, item: Item) {
		let len = match item {
			Item::Empty
			| Item::Text(..)
			| Item::Title2(..)
			| Item::Action2(..)
			| Item::Action3(..) => 0,
			Item::Label(ref label, _) | Item::Label2(ref label, _) | Item::Input(ref label, _) => {
				label.len()
			}
		};
		if len > self.pad {
			self.pad = len;
		}
		self.items.push(item);
	}

	pub fn add_bool<I: Into<String>>(&mut self, label: I, value: bool) {
		self.add_item(Item::Input(label.into(), Input::Bool(Bool::new(value))))
	}

	pub fn add_int<I: Into<String>>(
		&mut self,
		label: I,
		value: usize,
		min: usize,
		max: usize,
		step: usize,
	) {
		self.add_item(Item::Input(label.into(), Input::Int(Int::new(value, min, max, step))))
	}

	pub fn add_empty(&mut self) {
		self.add_item(Item::Empty)
	}

	pub fn add_title<I: Into<String>>(&mut self, label: I) {
		self.add_item(Item::Label(label.into(), true))
	}

	pub fn add_text<I: Into<String>>(&mut self, text: I) {
		self.add_item(Item::Text(text.into()))
	}

	pub fn add_font<I: Into<String>>(&mut self, label: I, value: config::Font) {
		self.add_item(Item::Input(label.into(), Input::Font(Font::new(value))))
	}

	pub fn add_action2(&mut self, action1: &'static str, action2: &'static str) {
		self.add_item(Item::Action2(action1, action2))
	}

	pub fn add_action3(
		&mut self,
		action1: &'static str,
		action2: &'static str,
		action3: &'static str,
	) {
		self.add_item(Item::Action3(action1, action2, action3))
	}

	pub fn add_rgb<I: Into<String>>(&mut self, label: I, rgb: config::Rgb) {
		self.add_item(Item::Input(label.into(), Input::Rgb(Rgb::new(rgb))))
	}

	pub fn add_key<I: Into<String>>(&mut self, label: I, value: Keycode) {
		self.add_item(Item::Input(label.into(), Input::Key(Key::new(value))))
	}

	pub fn add_info<I: Into<String>>(&mut self, label: I, info: I) {
		self.add_item(Item::Label2(label.into(), info.into()))
	}

	pub fn add_title2<I: Into<String>>(&mut self, title1: I, title2: I, size: usize) {
		self.add_item(Item::Title2(title1.into(), title2.into(), size))
	}

	pub fn add_cmd<I: Into<String>>(&mut self, label: I, cmd: Command) {
		self.add_item(Item::Input(label.into(), Input::Command(cmd)))
	}

	pub fn add_cmd2<I: Into<String>>(&mut self, label: I, cmd1: Command, cmd2: Command) {
		self.add_item(Item::Input(label.into(), Input::Command2(cmd1, cmd2)))
	}

	pub fn add_cmd_label2<I: Into<String>>(
		&mut self,
		label1: I,
		cmd1: Command,
		label2: I,
		cmd2: Command,
		pad: usize,
	) {
		self.add_item(Item::Input(
			label1.into(),
			Input::CommandLabel2(cmd1, label2.into(), pad, cmd2),
		))
	}

	pub fn add_device<I: Into<String>>(&mut self, label: I, device: Option<String>) {
		self.add_item(Item::Input(label.into(), Input::Device(Device::new(device))))
	}

	pub fn add_audio<I: Into<String>>(&mut self, label: I, device: Option<String>) {
		self.add_item(Item::Input(label.into(), Input::Audio(Audio::new(device))))
	}
}
