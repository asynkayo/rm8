use crate::nav::{Input, Item, Navigation};
use sdl2::keyboard::Keycode;

pub struct Remap {
	pos: usize,
	keys: Vec<Keycode>,
}

impl Remap {
	pub fn new(menu: &mut Navigation) -> Self {
		let mut keys = Vec::new();
		for (i, item) in menu.page_mut().items_mut().enumerate() {
			if let Item::Input(_, Input::Key(k)) = item {
				if i == 0 {
					k.focus();
				}
				keys.push(k.value());
			}
		}
		menu.page_mut().reset_cursor();
		menu.dirty();
		Self { pos: usize::MAX, keys }
	}

	pub fn remap(&mut self, menu: &mut Navigation, key: Keycode) -> bool {
		if self.pos == usize::MAX {
			self.pos = 0;
		} else {
			let mut items = menu.page_mut().items_mut().skip(self.pos);
			if let Some(Item::Input(_, Input::Key(k))) = items.next() {
				if self.keys[..self.pos].contains(&key) {
					k.exists();
				} else {
					k.unfocus();
					k.set_value(key);
					self.pos += 1;
					if let Some(Item::Input(_, Input::Key(k))) = items.next() {
						k.focus();
					}
				}
			}
		}
		menu.page_mut().reset_cursor();
		menu.dirty();
		self.pos >= self.keys.len()
	}

	pub fn abort(&mut self, menu: &mut Navigation) {
		for item in menu.page_mut().items_mut() {
			if let Item::Input(_, Input::Key(k)) = item {
				k.unfocus();
			}
		}
		menu.page_mut().reset_cursor();
		menu.dirty();
	}
}
