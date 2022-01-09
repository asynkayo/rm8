use sdl2::{
	joystick::{HatState, Joystick},
	keyboard::{Keycode, Mod},
	render::Canvas,
	video::Window,
	JoystickSubsystem,
};
use std::{
	cmp::Ordering,
	collections::HashMap,
	sync::{
		atomic::{self, AtomicBool},
		Arc,
	},
	thread,
	time::{self, Duration},
};

use crate::{
	config::{self, Command, Config},
	draw::{self, Context},
	font,
	m8::{self, M8},
	menu,
	menu_tools::{
		app_from_page, app_to_page, axes_from_page, buttons_from_page, clear_axes_page,
		clear_buttons_page, clear_hats_page, clear_joystick_subpages, hats_from_page,
		joystick_has_hats, m8_keys_from_page, m8_to_page, rm8_keys_from_page, rm8_to_page,
		selected_joystick_config, selected_joystick_guid, selected_joystick_id, theme_from_page,
		theme_to_page, update_axes_page, update_buttons_page, update_hats_page,
		update_joystick_pages,
	},
	nav::{Action, Direction, Edit, Navigation, Page},
	nav::{Input, Item},
	remap::Remap,
	value::Value,
};

pub const CONFIG_FILE: &str = "rm8.json";

const KEY_VEL_INC: u8 = 1 << 0;
const KEY_VEL_DEC: u8 = 1 << 1;
const KEY_OCT_INC: u8 = 1 << 2;
const KEY_OCT_DEC: u8 = 1 << 3;
const KEY_JAZZ: u8 = 1 << 4;
const KEY_FAST: u8 = 1 << 5;

pub struct App {
	config: Config,
	frame_ticks: time::Instant,
	config_ticks: time::Instant,
	joysticks: HashMap<String, Joystick>,
	menu: Navigation,
	joystick_page: Option<Page>,
	action: Action,
	in_config: bool,
	remap: Option<Remap>,
	keys: Value<u8>,
	running: Arc<AtomicBool>,
	defer: Option<Command>,
	fps: usize,
	fps_count: usize,
	fps_ticks: time::Instant,
}

impl App {
	pub fn new(running: Arc<AtomicBool>) -> Self {
		let mut config = Config::default();
		let _ = config.read(CONFIG_FILE);
		Self {
			frame_ticks: time::Instant::now(),
			config_ticks: time::Instant::now(),
			joysticks: HashMap::<String, Joystick>::new(),
			joystick_page: menu::build_joystick_page(),
			menu: Navigation::new(),
			action: Action::None,
			in_config: false,
			remap: None,
			keys: Value::<u8>::new(0),
			running,
			defer: None,
			fps: config.app.fps,
			fps_count: 0,
			fps_ticks: time::Instant::now(),
			config,
		}
	}

	pub fn build_menu(&mut self, m8: &M8) {
		menu::build_menu(&mut self.menu, m8, &self.config);
	}

	pub fn running(&self) -> bool {
		self.running.load(atomic::Ordering::SeqCst)
	}

	pub fn quit(&mut self) {
		self.running.store(false, atomic::Ordering::SeqCst);
	}

	pub fn config_mode(&self) -> bool {
		self.in_config
	}

	pub fn start_config_mode(&mut self) {
		self.menu.dirty();
		self.in_config = true;
	}

	pub fn cancel_config_mode(&mut self) {
		self.in_config = false;
	}

	pub fn remap_mode(&self) -> bool {
		self.remap.is_some()
	}

	pub fn remap(&mut self, keycode: Keycode) -> bool {
		if let Some(ref mut r) = self.remap {
			if r.remap(&mut self.menu, keycode) {
				r.abort(&mut self.menu);
				self.remap = None;
				return true;
			}
		}
		false
	}
	pub fn cancel_remap_mode(&mut self) {
		if let Some(mut r) = self.remap.take() {
			r.abort(&mut self.menu);
			self.remap = None;
		}
	}

	pub fn config(&self) -> &Config {
		&self.config
	}

	pub fn config_mut(&mut self) -> &mut Config {
		&mut self.config
	}

	pub fn handle_key(&mut self, m8: &mut M8, keycode: Keycode, keymod: Mod, clear: bool) {
		let f = if clear { Value::clr_bit } else { Value::set_bit };
		if clear && *m8.keyjazz && self.config.keyjazz.contains_key(&config::Keycode(keycode)) {
			m8.set_note_off()
		}
		if keycode == *self.config.m8.up {
			f(&mut m8.keys, m8::KEY_UP);
		} else if keycode == *self.config.m8.down {
			f(&mut m8.keys, m8::KEY_DOWN);
		} else if keycode == *self.config.m8.left {
			f(&mut m8.keys, m8::KEY_LEFT);
		} else if keycode == *self.config.m8.right {
			f(&mut m8.keys, m8::KEY_RIGHT);
		} else if keycode == *self.config.m8.edit {
			f(&mut m8.keys, m8::KEY_EDIT);
		} else if keycode == *self.config.m8.option {
			f(&mut m8.keys, m8::KEY_OPTION);
		} else if keycode == *self.config.m8.shift {
			f(&mut m8.keys, m8::KEY_SHIFT);
		} else if keycode == *self.config.m8.play {
			f(&mut m8.keys, m8::KEY_PLAY);
		} else if keycode == *self.config.rm8.octave_minus {
			f(&mut self.keys, KEY_OCT_DEC);
		} else if keycode == *self.config.rm8.octave_plus {
			f(&mut self.keys, KEY_OCT_INC);
		} else if keycode == *self.config.rm8.velocity_minus {
			if clear {
				f(&mut self.keys, KEY_VEL_DEC | KEY_FAST);
			} else {
				f(
					&mut self.keys,
					KEY_VEL_DEC
						| if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
							KEY_FAST
						} else {
							0
						},
				);
			}
		} else if keycode == *self.config.rm8.velocity_plus {
			if clear {
				f(&mut self.keys, KEY_VEL_INC | KEY_FAST);
			} else {
				f(
					&mut self.keys,
					KEY_VEL_INC
						| if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
							KEY_FAST
						} else {
							0
						},
				);
			}
		}
	}

	pub fn button_cmd(
		&self,
		joystick_id: u32,
		button_id: u8,
		release: bool,
	) -> Option<(Command, bool)> {
		if let Some(page) = self.menu.find('J') {
			if selected_joystick_id(page) == joystick_id {
				if let Some(joystick) = selected_joystick_config(page, &self.config) {
					if let Some(cmd) = joystick.buttons.get(&button_id) {
						return Some((*cmd, release));
					}
				}
			}
		}
		None
	}

	pub fn axis_cmd(&self, joystick_id: u32, axis_id: u8, value: i16) -> Option<(Command, bool)> {
		if let Some(page) = self.menu.find('J') {
			if selected_joystick_id(page) == joystick_id {
				if let Some(joystick) = selected_joystick_config(page, &self.config) {
					if let Some(axis) = joystick.axes.get(&axis_id) {
						if value.saturating_abs() as usize <= axis.sensibility {
							return Some((Command::None, true));
						}
						match value.cmp(&0) {
							Ordering::Less => {
								if axis.negative != Command::None {
									return Some((axis.negative, false));
								}
							}
							Ordering::Greater => {
								if value as usize > axis.sensibility {
									return Some((axis.positive, false));
								}
							}
							_ => {}
						}
					}
				}
			}
		}
		None
	}

	pub fn hat_cmd(&self, joystick_id: u32, state: HatState) -> Option<(Command, bool)> {
		if let Some(page) = self.menu.find('J') {
			if selected_joystick_id(page) == joystick_id {
				if let Some(joystick) = selected_joystick_config(page, &self.config) {
					if let Some(hats) = &joystick.hats {
						if let Some(hat) = hats.get(&0) {
							let cmd = match state {
								HatState::Up => hat.up,
								HatState::Down => hat.down,
								HatState::Left => hat.left,
								HatState::Right => hat.right,
								HatState::LeftUp => hat.left_up,
								HatState::LeftDown => hat.left_down,
								HatState::RightUp => hat.right_up,
								HatState::RightDown => hat.right_down,
								HatState::Centered => Command::None,
							};
							return Some((cmd, cmd == Command::None));
						}
					}
				}
			}
		}
		None
	}

	pub fn handle_cmd(&mut self, m8: &mut M8, cmd: Option<(Command, bool)>) {
		if let Some((cmd, clear)) = cmd {
			let f = if clear { Value::clr_bit } else { Value::set_bit };
			match cmd {
				Command::Up => f(&mut m8.keys, m8::KEY_UP),
				Command::Down => f(&mut m8.keys, m8::KEY_DOWN),
				Command::Left => f(&mut m8.keys, m8::KEY_LEFT),
				Command::Right => f(&mut m8.keys, m8::KEY_RIGHT),
				Command::Edit => f(&mut m8.keys, m8::KEY_EDIT),
				Command::r#Option => f(&mut m8.keys, m8::KEY_OPTION),
				Command::Shift => f(&mut m8.keys, m8::KEY_SHIFT),
				Command::Play => f(&mut m8.keys, m8::KEY_PLAY),

				Command::Keyjazz => f(&mut m8.keys, KEY_JAZZ),
				Command::VelocityMinus => f(&mut m8.keys, KEY_VEL_DEC),
				Command::VelocityPlus => f(&mut m8.keys, KEY_VEL_INC),
				Command::OctaveMinus => f(&mut m8.keys, KEY_OCT_DEC),
				Command::OctavePlus => f(&mut m8.keys, KEY_OCT_INC),
				Command::Config => self.start_config_mode(),
				Command::Escape | Command::Fullscreen | Command::Reset | Command::ResetFull => {
					self.defer.replace(cmd);
				}
				Command::None => m8.keys.clr_bit(m8::KEY_DIR),
			}
		}
	}

	pub fn add_joystick(&mut self, joystick_subsystem: &JoystickSubsystem, which: u32) {
		if self.menu.main_page_short_name() == 'J' {
			self.menu.dirty();
		}
		if let Ok(j) = joystick_subsystem.open(which) {
			if self.joysticks.is_empty() {
				if let Some(page) = self.joystick_page.take() {
					self.joystick_page.replace(self.menu.replace('J', page));
				}
			}
			self.joysticks.insert(j.guid().string(), j);
			update_joystick_pages(
				&mut self.menu,
				joystick_subsystem,
				&self.joysticks,
				&self.config,
			);
		}
	}

	pub fn rem_joystick(&mut self, joystick_subsystem: &JoystickSubsystem, which: u32) {
		self.joysticks.retain(|_, j| j.instance_id() != which);
		if self.joysticks.is_empty() {
			if let Some(page) = self.joystick_page.take() {
				self.joystick_page.replace(self.menu.replace('J', page));
			}
		}
		if self.menu.main_page_short_name() == 'J' {
			self.menu.dirty();
			self.menu.main_page_reset();
		}
		update_joystick_pages(&mut self.menu, joystick_subsystem, &self.joysticks, &self.config);
	}

	pub fn process_key(&mut self, m8: &mut M8) {
		let now = time::Instant::now();
		if now - self.config_ticks > Duration::from_millis(self.config.app.key_sensibility) {
			if self.in_config {
				self.config_ticks = now;
				if m8.keys.tst_bit(m8::KEY_UP) {
					if m8.keys.tst_bit(m8::KEY_SHIFT) {
						self.menu.navigate(Direction::Above);
						m8.keys.clr_bit(m8::KEY_DIR);
					} else if m8.keys.tst_bit(m8::KEY_EDIT) {
						self.action.map(self.menu.edit_item(Edit::Next(true)));
					} else {
						self.menu.cursor_move(Direction::Above);
					}
				} else if m8.keys.tst_bit(m8::KEY_DOWN) {
					if m8.keys.tst_bit(m8::KEY_SHIFT) {
						self.menu.navigate(Direction::Below);
						m8.keys.clr_bit(m8::KEY_DIR);
					} else if m8.keys.tst_bit(m8::KEY_EDIT) {
						self.action.map(self.menu.edit_item(Edit::Prev(true)));
					} else {
						self.menu.cursor_move(Direction::Below);
					}
				} else if m8.keys.tst_bit(m8::KEY_LEFT) {
					if m8.keys.tst_bit(m8::KEY_SHIFT) {
						self.menu.navigate(Direction::Left);
						m8.keys.clr_bit(m8::KEY_DIR);
					} else if m8.keys.tst_bit(m8::KEY_EDIT) {
						self.action.map(self.menu.edit_item(Edit::Prev(false)));
					} else {
						self.menu.cursor_move(Direction::Left);
					}
				} else if m8.keys.tst_bit(m8::KEY_RIGHT) {
					if m8.keys.tst_bit(m8::KEY_SHIFT) {
						self.menu.navigate(Direction::Right);
						m8.keys.clr_bit(m8::KEY_DIR);
					} else if m8.keys.tst_bit(m8::KEY_EDIT) {
						self.action.map(self.menu.edit_item(Edit::Next(false)));
					} else {
						self.menu.cursor_move(Direction::Right);
					}
				} else if m8.keys.tst_bit(m8::KEY_EDIT) {
					let cmd = if m8.keys.tst_bit(m8::KEY_OPTION) {
						self.menu.edit_item(Edit::Reset)
					} else {
						self.menu.edit_item(Edit::Click)
					};
					if cmd != Action::None {
						m8.keys.clr_bit(m8::KEY_EDIT);
					}
					self.action.map(cmd);
					m8.keys.clr_bit(m8::KEY_DIR);
				}
			} else if self.keys.tst_bit(KEY_OCT_DEC) {
				m8.dec_octave();
			} else if self.keys.tst_bit(KEY_OCT_INC) {
				m8.inc_octave();
			} else if self.keys.tst_bit(KEY_VEL_DEC) {
				m8.dec_velocity(self.keys.tst_bit(KEY_FAST));
			} else if self.keys.tst_bit(KEY_VEL_INC) {
				m8.inc_velocity(self.keys.tst_bit(KEY_FAST));
			} else if self.keys.tst_bit(KEY_JAZZ) {
				m8.keyjazz.toggle();
			}
		}
		self.keys.set(0);
	}

	fn action_modified(
		&mut self,
		canvas: &mut Canvas<Window>,
		m8: &mut M8,
		joystick_subsystem: &JoystickSubsystem,
	) -> Result<(), String> {
		let mut dirty = false;
		let page = self.menu.page();
		match page.short_name() {
			'C' => {
				let old_zoom = self.config.app.zoom;
				self.config.app = app_from_page(page);
				if self.config.app.zoom != old_zoom {
					draw::zoom_window(canvas.window_mut(), self.config.app.zoom);
				}
				if let Some(Item::Input(_, Input::Device(d))) = page.items().nth(7) {
					let device = d.value();
					if device != m8.device_name().as_deref() {
						if let Some(dev) = device {
							if let Ok(new_m8) = M8::open(dev) {
								*m8 = new_m8;
								m8.enable_and_reset_display()?;
							}
						}
					}
				}
				m8.set_reconnect(self.config.app.reconnect);
				m8.keyjazz.set(!self.config.overlap);
				dirty = true;
			}
			'T' => {
				self.config.theme = theme_from_page(page);
				dirty = true;
			}
			'K' => {
				self.config.m8 = m8_keys_from_page(page);
			}
			'R' => {
				self.config.rm8 = rm8_keys_from_page(page);
			}
			'J' => {
				dirty = true;
				update_joystick_pages(
					&mut self.menu,
					joystick_subsystem,
					&self.joysticks,
					&self.config,
				);
			}
			'B' => {
				if let Some(guid) = selected_joystick_guid(&self.menu) {
					let cfg = self.config.joysticks.entry(guid.into()).or_default();
					cfg.buttons = buttons_from_page(page);
				}
			}
			'A' => {
				if let Some(guid) = selected_joystick_guid(&self.menu) {
					let cfg = self.config.joysticks.entry(guid.into()).or_default();
					cfg.axes = axes_from_page(page);
				}
			}
			'H' => {
				if let Some(guid) = selected_joystick_guid(&self.menu) {
					if joystick_has_hats(self.menu.main_page()) {
						let cfg = self.config.joysticks.entry(guid.into()).or_default();
						cfg.hats = Some(hats_from_page(page));
					}
				}
			}
			_ => {}
		}
		if dirty {
			self.menu.dirty();
		}
		Ok(())
	}

	fn action_save(&mut self, config_file: Option<&str>) -> Result<(), String> {
		let page = self.menu.page();
		match page.short_name() {
			'C' => {
				if let Some(sub) = page.find('T') {
					self.config.theme = theme_from_page(sub);
				}
				self.config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
			}
			'T' => {
				self.config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
			}
			'K' => {
				if let Some(sub) = page.find('R') {
					self.config.rm8 = rm8_keys_from_page(sub);
				}
				self.config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
			}
			'R' => {
				self.config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
			}
			'J' => {
				if let Some(guid) = selected_joystick_guid(&self.menu) {
					let cfg = self.config.joysticks.entry(guid.into()).or_default();
					if let Some(sub) = page.find('B') {
						cfg.buttons = buttons_from_page(sub);
					}
					if let Some(sub) = page.find('A') {
						cfg.axes = axes_from_page(sub);
					}
					if let Some(sub) = page.find('H') {
						if joystick_has_hats(self.menu.main_page()) {
							cfg.hats = Some(hats_from_page(sub));
						}
					}
					self.config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
				}
			}
			'B' => {
				if selected_joystick_guid(&self.menu).is_some() {
					self.config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
				}
			}
			'A' => {
				if selected_joystick_guid(&self.menu).is_some() {
					self.config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
				}
			}
			'H' => {
				if selected_joystick_guid(&self.menu).is_some()
					&& joystick_has_hats(self.menu.main_page())
				{
					self.config.write(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
				}
			}
			_ => {}
		}
		Ok(())
	}

	fn action_reset(
		&mut self,
		config_file: Option<&str>,
		joystick_subsystem: &JoystickSubsystem,
	) -> Result<(), String> {
		let mut dirty = true;
		let mut cfg = Config::default();
		cfg.read(config_file.as_deref().unwrap_or(CONFIG_FILE))?;
		match self.menu.page().short_name() {
			'C' => {
				let mut page = self.menu.page_mut();
				self.config.app = cfg.app;
				app_to_page(&mut page, &self.config);
			}
			'T' => {
				let mut page = self.menu.page_mut();
				self.config.theme = cfg.theme;
				theme_to_page(&mut page, &self.config);
			}
			'K' => {
				let mut page = self.menu.page_mut();
				self.config.m8 = cfg.m8;
				m8_to_page(&mut page, &self.config);
			}
			'R' => {
				let mut page = self.menu.page_mut();
				self.config.rm8 = cfg.rm8;
				rm8_to_page(&mut page, &self.config);
			}
			'J' => {
				if let Some(guid) = selected_joystick_guid(&self.menu) {
					let old = self.config.joysticks.get_mut(guid);
					let new = cfg.joysticks.remove(guid);
					if let (Some(old), Some(new)) = (old, new) {
						*old = new;
						update_joystick_pages(
							&mut self.menu,
							joystick_subsystem,
							&self.joysticks,
							&self.config,
						);
					} else {
						clear_joystick_subpages(&mut self.menu);
					}
				}
			}
			'A' => {
				if let Some(guid) = selected_joystick_guid(&self.menu) {
					let old = self.config.joysticks.get_mut(guid);
					let new = cfg.joysticks.remove(guid);
					let mut page = self.menu.page_mut();
					if let (Some(old), Some(new)) = (old, new) {
						update_axes_page(&mut page, &new);
						old.axes = new.axes;
					} else {
						clear_axes_page(&mut page);
					}
				}
			}
			'B' => {
				if let Some(guid) = selected_joystick_guid(&self.menu) {
					let old = self.config.joysticks.get_mut(guid);
					let new = cfg.joysticks.remove(guid);
					let mut page = self.menu.page_mut();
					if let (Some(old), Some(new)) = (old, new) {
						update_buttons_page(&mut page, &new);
						old.buttons = new.buttons;
					} else {
						clear_buttons_page(&mut page);
					}
				}
			}
			'H' => {
				if let Some(guid) = selected_joystick_guid(&self.menu) {
					let old = self.config.joysticks.get_mut(guid);
					let new = cfg.joysticks.remove(guid);
					let mut page = self.menu.page_mut();
					if let (Some(old), Some(new)) = (old, new) {
						update_hats_page(&mut page, &new);
						old.hats = new.hats;
					} else {
						clear_hats_page(&mut page);
					}
				}
			}
			_ => {
				dirty = false;
			}
		}
		if dirty {
			self.menu.dirty();
		}
		Ok(())
	}

	pub fn process_action(
		&mut self,
		canvas: &mut Canvas<Window>,
		m8: &mut M8,
		joystick_subsystem: &JoystickSubsystem,
		config_file: &Option<String>,
	) -> Result<(), String> {
		match self.action {
			Action::Modified => self.action_modified(canvas, m8, joystick_subsystem)?,
			Action::Do("SAVE") => self.action_save(config_file.as_deref())?,
			Action::Do("RESET") => self.action_reset(config_file.as_deref(), joystick_subsystem)?,
			Action::Do("REMAP") => self.remap = Some(Remap::new(&mut self.menu)),
			Action::Do(_) => unimplemented!(),
			Action::None => {}
		}
		self.action.reset();
		Ok(())
	}

	pub fn render(&mut self, ctx: &mut Context<'_, '_, '_>) -> Result<(), String> {
		self.menu.draw(ctx)
	}

	pub fn render_fps(&mut self, ctx: &mut Context<'_, '_, '_>) -> Result<(), String> {
		if self.config.app.show_fps {
			self.fps_count += 1;
			let now = time::Instant::now();
			if now - self.fps_ticks > Duration::from_secs(5) {
				self.fps_ticks = now;
				self.fps = self.fps_count;
				self.fps_count = 0;
			}
			let fg = self.config.theme.text_default;
			ctx.draw_rect(
				(0, 0, font::width(6) as u32, draw::LINE_HEIGHT as u32),
				self.config.theme.screen,
			)?;
			ctx.draw_str(&format!("{:3} fps", self.fps / 5), 0, 0, fg, fg)?;
		}
		Ok(())
	}

	pub fn sync(&mut self) -> bool {
		let now = time::Instant::now();
		if now - self.frame_ticks > Duration::from_millis(15) {
			self.frame_ticks = now;
			return true;
		}
		let fps_sleep = (1.0 / self.config.app.fps as f64 * 1000.0) as u64;
		thread::sleep(Duration::from_millis(fps_sleep));
		false
	}

	pub fn escape_command(
		&mut self,
		m8: &mut M8,
		canvas: &mut Canvas<Window>,
	) -> Result<(), String> {
		if draw::is_fullscreen(canvas) {
			draw::toggle_fullscreen(canvas)?;
		} else if self.config_mode() {
			if self.remap_mode() {
				self.cancel_remap_mode();
			} else {
				self.cancel_config_mode();
			}
			m8.refresh();
		} else {
			self.running.store(false, atomic::Ordering::SeqCst);
		}
		Ok(())
	}

	pub fn handle_defer(&mut self, m8: &mut M8, canvas: &mut Canvas<Window>) -> Result<(), String> {
		match self.defer.take() {
			Some(Command::Escape) => self.escape_command(m8, canvas)?,
			Some(Command::Fullscreen) => draw::toggle_fullscreen(canvas)?,
			Some(Command::Reset) => m8.reset(false)?,
			Some(Command::ResetFull) => m8.reset(true)?,
			Some(_) | None => {}
		}
		Ok(())
	}
}
