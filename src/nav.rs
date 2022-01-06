use crate::{
	draw::{Context, LINE_HEIGHT},
	font, m8,
};

pub use crate::{
	nav_item::{Action, Direction, Edit, Input, Item},
	nav_page::Page,
};

const MENU_X: i32 = 288;
const MENU_Y: i32 = 200;

#[derive(Debug)]
pub struct Navigation {
	pages: Vec<Page>,
	page: (usize, isize),
	changed: bool,
}

impl Navigation {
	pub fn new() -> Self {
		Self { pages: vec![], page: (0, 0), changed: true }
	}

	pub fn find(&self, short: char) -> Option<&Page> {
		for page in self.pages.iter() {
			if page.short_name() == short {
				return Some(page);
			}
		}
		None
	}

	pub fn find_mut(&mut self, short: char) -> Option<&mut Page> {
		for page in self.pages.iter_mut() {
			if page.short_name() == short {
				return Some(page);
			}
		}
		None
	}

	pub fn add_page(&mut self, mut page: Page) {
		page.reset_cursor();
		self.pages.push(page);
	}

	pub fn page_mut(&mut self) -> &mut Page {
		self.pages[self.page.0].get_mut(self.page.1).unwrap()
	}

	pub fn page(&self) -> &Page {
		self.pages[self.page.0].get(self.page.1).unwrap()
	}

	pub fn main_page(&self) -> &Page {
		&self.pages[self.page.0]
	}

	pub fn edit_item(&mut self, edit: Edit) -> Action {
		self.page_mut().edit_item(edit)
	}

	pub fn replace(&mut self, short: char, mut page: Page) -> Page {
		for p in self.pages.iter_mut() {
			if p.short_name() == short {
				page.reset_cursor();
				return std::mem::replace(p, page);
			}
		}
		page
	}

	pub fn navigate(&mut self, direction: Direction) {
		match direction {
			Direction::Left => {
				if self.page.0 > 0 {
					self.page.0 -= 1;
					self.page.1 = 0;
					self.changed = true;
				}
			}
			Direction::Right => {
				if self.page.0 + 1 < self.pages.len() {
					self.page.0 += 1;
					self.page.1 = 0;
					self.changed = true;
				}
			}
			Direction::Above => {
				if self.pages[self.page.0].get(self.page.1 + 1).is_some() {
					self.page.1 += 1;
					self.changed = true;
				}
			}
			Direction::Below => {
				if self.pages[self.page.0].get(self.page.1 - 1).is_some() {
					self.page.1 -= 1;
					self.changed = true;
				}
			}
		}
	}

	pub fn cursor_move(&mut self, direction: Direction) {
		self.page_mut().cursor_move(direction);
	}

	pub fn main_page_short_name(&self) -> char {
		self.pages[self.page.0].short_name()
	}

	pub fn main_page_reset(&mut self) {
		self.page.1 = 0
	}

	pub fn dirty(&mut self) {
		self.changed = true
	}

	fn draw_sub_menu(
		&self,
		ctx: &mut Context<'_, '_, '_>,
		sub: &[Page],
		x: i32,
		line_height: i32,
		selected: Option<usize>,
	) -> Result<(), String> {
		let fg = ctx.theme.text_info;
		let hl = ctx.theme.text_title;
		let mut y = MENU_Y + line_height;
		for (i, s) in sub.iter().enumerate() {
			match selected {
				Some(sel) if sel == i => ctx.draw_char(s.short_name() as u8, x, y, hl, hl)?,
				_ => ctx.draw_char(s.short_name() as u8, x, y, fg, fg)?,
			}
			y += line_height;
		}
		Ok(())
	}

	fn draw_menu(&self, ctx: &mut Context<'_, '_, '_>) -> Result<(), String> {
		let fg = ctx.theme.text_info;
		let hl = ctx.theme.text_title;
		let mut x = MENU_X;
		for (i, page) in self.pages.iter().enumerate() {
			let page_y = self.page.1.saturating_abs() as usize;
			let sel_above =
				if self.page.0 == i && self.page.1 > 0 { Some(page_y - 1) } else { None };
			let sel_below =
				if self.page.0 == i && self.page.1 < 0 { Some(page_y - 1) } else { None };
			self.draw_sub_menu(ctx, self.pages[i].above(), x, -(LINE_HEIGHT as i32), sel_above)?;
			self.draw_sub_menu(ctx, self.pages[i].below(), x, LINE_HEIGHT as i32, sel_below)?;
			if self.page.0 == i && self.page.1 == 0 {
				ctx.draw_char(page.short_name() as u8, x, MENU_Y, hl, hl)?;
			} else {
				ctx.draw_char(page.short_name() as u8, x, MENU_Y, fg, fg)?;
			}
			x += font::width(0);
		}
		Ok(())
	}

	pub fn draw(&mut self, ctx: &mut Context<'_, '_, '_>) -> Result<(), String> {
		if self.changed {
			ctx.draw_rect((0, 0, m8::SCREEN_WIDTH, m8::SCREEN_HEIGHT), ctx.theme.screen)?;
			self.changed = false;
		}
		self.page_mut().draw(ctx)?;
		self.draw_menu(ctx)
	}
}
