use crate::{
	config::{self, Axis, Command, Config, HatConfig, JoystickConfig},
	nav::{Input, Item, Navigation, Page},
};
use sdl2::{joystick::Joystick, JoystickSubsystem};
use std::{collections::HashMap, fmt::Write};

pub fn selected_joystick_id(page: &Page) -> u32 {
	return usize_from_item(page.items().nth(1)) as u32;
}

pub fn selected_joystick_config<'a>(page: &Page, config: &'a Config) -> Option<&'a JoystickConfig> {
	if let Some(Item::Text(ref guid)) = page.items().nth(4) {
		if guid.len() > 5 {
			return config.joysticks.get(&guid[5..]);
		}
	}
	None
}

pub fn selected_joystick_guid(nav: &Navigation) -> Option<&str> {
	if let Some(page) = nav.find('J') {
		if let Some(Item::Text(ref guid)) = page.items().nth(4) {
			if guid.len() > 5 {
				return Some(&guid[5..]);
			}
		}
	}
	None
}

pub fn clear_joystick_subpages(menu: &mut Navigation) {
	if let Some(page) = menu.find_mut('A') {
		clear_axes_page(page);
	}
	if let Some(page) = menu.find_mut('B') {
		clear_buttons_page(page);
	}
	if let Some(page) = menu.find_mut('H') {
		clear_hats_page(page);
	}
}

pub fn update_joystick_pages(
	menu: &mut Navigation,
	joystick_subsystem: &JoystickSubsystem,
	joysticks: &HashMap<String, Joystick>,
	config: &Config,
) {
	if let Some(page) = menu.find_mut('J') {
		update_joystick_page(page, joystick_subsystem, joysticks);
		if let Some(joystick) = selected_joystick_config(page, config) {
			update_joystick_subpages(page, joystick);
		}
	}
}

fn update_joystick_page(
	page: &mut Page,
	joystick_subsystem: &JoystickSubsystem,
	joysticks: &HashMap<String, Joystick>,
) {
	let mut items = page.items_mut();
	usize_to_label(items.next(), joysticks.len());
	let id = if let Some(Item::Input(_, Input::Int(i))) = items.next() {
		i.set_max(joysticks.len() - 1);
		i.value()
	} else {
		0
	};
	items.next();
	if let Ok(guid) = joystick_subsystem.device_guid(id as u32) {
		let guid = guid.string();
		if let Some(joystick) = joysticks.get(&guid) {
			str_to_label(items.next(), joystick.name());
			str_to_label(items.next(), format!("GUID {}", guid));
			usize_to_label(items.next(), joystick.num_axes() as usize);
			usize_to_label(items.next(), joystick.num_buttons() as usize);
			usize_to_label(items.next(), joystick.num_hats() as usize);
		}
	}
}

fn update_joystick_subpages(page: &mut Page, joystick: &JoystickConfig) {
	if let Some(page) = page.find_mut('A') {
		update_axes_page(page, joystick);
	}
	if let Some(page) = page.find_mut('B') {
		update_buttons_page(page, joystick);
	}
	if let Some(page) = page.find_mut('H') {
		update_hats_page(page, joystick);
	}
}

pub fn update_buttons_page(page: &mut Page, config: &JoystickConfig) {
	clear_buttons_page(page);
	for (i, item) in page.items_mut().enumerate() {
		if let Item::Input(_, Input::CommandLabel2(c1, .., c2)) = item {
			if let Some(b) = config.buttons.get(&(i as u8)) {
				*c1 = *b;
			}
			if let Some(b) = config.buttons.get(&(i as u8 + 10)) {
				*c2 = *b;
			}
		}
	}
}

pub fn update_axes_page(page: &mut Page, config: &JoystickConfig) {
	clear_axes_page(page);
	let mut i = 0;
	let mut items = page.items_mut().skip(1);
	while let Some(item) = items.next() {
		if let Item::Input(_, Input::Command2(c1, c2)) = item {
			if let Some(Axis { negative, positive, .. }) = config.axes.get(&(i as u8)) {
				*c1 = *negative;
				*c2 = *positive;
			} else {
				*c1 = Command::None;
				*c2 = Command::None;
			}
		}
		if let Some(Item::Input(_, Input::Int(n))) = items.next() {
			if let Some(Axis { sensibility, .. }) = config.axes.get(&(i as u8)) {
				n.set_value(*sensibility);
			} else {
				n.set_value(0);
			}
		}
		i += 1;
	}
}

pub fn update_hats_page(page: &mut Page, config: &JoystickConfig) {
	clear_hats_page(page);
	if let Some(ref hats) = config.hats {
		let mut items = page.items_mut();
		if let Some(h) = &hats.get(&0) {
			cmd_to_item(items.next(), h.up);
			cmd_to_item(items.next(), h.down);
			cmd_to_item(items.next(), h.left);
			cmd_to_item(items.next(), h.right);
			cmd_to_item(items.next(), h.left_up);
			cmd_to_item(items.next(), h.right_up);
			cmd_to_item(items.next(), h.left_down);
			cmd_to_item(items.next(), h.right_down);
		}
	}
}

pub fn clear_buttons_page(page: &mut Page) {
	for item in page.items_mut() {
		if let Item::Input(_, Input::CommandLabel2(c1, .., c2)) = item {
			*c1 = Command::None;
			*c2 = Command::None;
		}
	}
}

pub fn clear_axes_page(page: &mut Page) {
	for item in page.items_mut().skip(1) {
		match item {
			Item::Input(_, Input::Command2(c1, c2)) => {
				*c1 = Command::None;
				*c2 = Command::None;
			}
			Item::Input(_, Input::Int(n)) => {
				n.set_value(0);
			}
			_ => {}
		}
	}
}

pub fn clear_hats_page(page: &mut Page) {
	for item in page.items_mut() {
		if let Item::Input(_, Input::Command(c)) = item {
			*c = Command::None;
		}
	}
}

pub fn buttons_from_page(page: &Page) -> HashMap<u8, Command> {
	let mut buttons = HashMap::new();
	for (i, item) in page.items().enumerate() {
		if let Item::Input(_, Input::CommandLabel2(c1, .., c2)) = item {
			buttons.insert(i as u8, *c1);
			buttons.insert(i as u8 + 10, *c2);
		}
	}
	buttons
}

pub fn axes_from_page(page: &Page) -> HashMap<u8, Axis> {
	let mut axes = HashMap::new();
	let mut axis = (Command::None, Command::None);
	for (i, item) in page.items().skip(1).enumerate() {
		match item {
			Item::Input(_, Input::Command2(c1, c2)) => {
				axis = (*c1, *c2);
			}
			Item::Input(_, Input::Int(v)) => {
				axes.insert(i as u8 / 2, Axis::new(axis.0, axis.1, v.value()));
			}
			_ => {}
		}
	}
	axes
}

pub fn hats_from_page(page: &Page) -> HashMap<u32, HatConfig> {
	let mut items = page.items();
	HashMap::from([(
		0,
		HatConfig {
			up: cmd_from_item(items.next()),
			down: cmd_from_item(items.next()),
			left: cmd_from_item(items.next()),
			right: cmd_from_item(items.next()),
			left_up: cmd_from_item(items.next()),
			right_up: cmd_from_item(items.next()),
			left_down: cmd_from_item(items.next()),
			right_down: cmd_from_item(items.next()),
		},
	)])
}

pub fn joystick_has_hats(page: &Page) -> bool {
	if let Some(Item::Label2(_, value)) = page.items().nth(7) {
		return value != "0";
	}
	false
}

pub fn m8_to_page(page: &mut Page, config: &config::Config) {
	let mut items = page.items_mut();
	key_to_item(items.next(), config.m8.up);
	key_to_item(items.next(), config.m8.down);
	key_to_item(items.next(), config.m8.left);
	key_to_item(items.next(), config.m8.right);
	key_to_item(items.next(), config.m8.edit);
	key_to_item(items.next(), config.m8.option);
	key_to_item(items.next(), config.m8.shift);
	key_to_item(items.next(), config.m8.play);
}

pub fn rm8_to_page(page: &mut Page, config: &config::Config) {
	let mut items = page.items_mut();
	key_to_item(items.next(), config.rm8.keyjazz);
	key_to_item(items.next(), config.rm8.velocity_minus);
	key_to_item(items.next(), config.rm8.velocity_plus);
	key_to_item(items.next(), config.rm8.octave_minus);
	key_to_item(items.next(), config.rm8.octave_plus);
}

pub fn rm8_keys_from_page(page: &Page) -> config::RM8KeyboardConfig {
	let mut items = page.items();
	config::RM8KeyboardConfig {
		keyjazz: key_from_item(items.next()),
		velocity_minus: key_from_item(items.next()),
		velocity_plus: key_from_item(items.next()),
		octave_minus: key_from_item(items.next()),
		octave_plus: key_from_item(items.next()),
	}
}

pub fn m8_keys_from_page(page: &Page) -> config::M8KeyboardConfig {
	let mut items = page.items();
	config::M8KeyboardConfig {
		up: key_from_item(items.next()),
		down: key_from_item(items.next()),
		left: key_from_item(items.next()),
		right: key_from_item(items.next()),
		edit: key_from_item(items.next()),
		option: key_from_item(items.next()),
		shift: key_from_item(items.next()),
		play: key_from_item(items.next()),
	}
}

pub fn theme_from_page(page: &Page) -> config::ThemeConfig {
	let mut items = page.items();
	config::ThemeConfig {
		text_default: rgb_from_item(items.next()),
		text_value: rgb_from_item(items.next()),
		text_title: rgb_from_item(items.next()),
		text_info: rgb_from_item(items.next()),
		cursor: rgb_from_item(items.next()),
		screen: rgb_from_item(items.next()),
		velocity_fg: rgb_from_item(items.next()),
		velocity_bg: rgb_from_item(items.next()),
		octave_fg: rgb_from_item(items.next()),
		octave_bg: rgb_from_item(items.next()),
	}
}

pub fn theme_to_page(page: &mut Page, config: &Config) {
	let mut items = page.items_mut();
	rgb_to_item(items.next(), config.theme.text_default);
	rgb_to_item(items.next(), config.theme.text_value);
	rgb_to_item(items.next(), config.theme.text_title);
	rgb_to_item(items.next(), config.theme.text_info);
	rgb_to_item(items.next(), config.theme.cursor);
	rgb_to_item(items.next(), config.theme.screen);
	rgb_to_item(items.next(), config.theme.velocity_fg);
	rgb_to_item(items.next(), config.theme.velocity_bg);
	rgb_to_item(items.next(), config.theme.octave_fg);
	rgb_to_item(items.next(), config.theme.octave_bg);
}

pub fn app_to_page(page: &mut Page, config: &Config) {
	let mut items = page.items_mut();
	bool_to_item(items.next(), config.app.fullscreen);
	int_to_item(items.next(), config.app.zoom as usize);
	font_to_item(items.next(), config.app.font);
	int_to_item(items.next(), config.app.key_sensibility as usize);
}

pub fn app_from_page(page: &Page) -> config::AppConfig {
	let mut items = page.items();
	config::AppConfig {
		fullscreen: bool_from_item(items.next()),
		zoom: int_from_item(items.next()) as u32,
		font: font_from_item(items.next()),
		key_sensibility: int_from_item(items.next()) as u64,
	}
}

fn cmd_to_item(item: Option<&mut Item>, cmd: Command) {
	if let Some(Item::Input(_, Input::Command(c))) = item {
		*c = cmd;
	}
}

fn cmd_from_item(item: Option<&Item>) -> Command {
	if let Some(Item::Input(_, Input::Command(c))) = item {
		*c
	} else {
		Command::None
	}
}

fn int_to_item(item: Option<&mut Item>, i: usize) {
	if let Some(Item::Input(_, Input::Int(value))) = item {
		value.set_value(i);
	}
}

fn int_from_item(item: Option<&Item>) -> usize {
	if let Some(Item::Input(_, Input::Int(i))) = item {
		i.value()
	} else {
		0
	}
}

fn font_to_item(item: Option<&mut Item>, f: config::Font) {
	if let Some(Item::Input(_, Input::Font(value))) = item {
		value.set_value(f)
	}
}

fn font_from_item(item: Option<&Item>) -> config::Font {
	if let Some(Item::Input(_, Input::Font(f))) = item {
		f.value()
	} else {
		config::Font::Uppercase
	}
}

fn bool_to_item(item: Option<&mut Item>, b: bool) {
	if let Some(Item::Input(_, Input::Bool(value))) = item {
		value.set_value(b);
	}
}

fn bool_from_item(item: Option<&Item>) -> bool {
	if let Some(Item::Input(_, Input::Bool(b))) = item {
		b.value()
	} else {
		false
	}
}

fn rgb_to_item(item: Option<&mut Item>, value: config::Rgb) {
	if let Some(Item::Input(_, Input::Rgb(rgb))) = item {
		rgb.set_value(value);
	}
}

fn rgb_from_item(item: Option<&Item>) -> config::Rgb {
	if let Some(Item::Input(_, Input::Rgb(rgb))) = item {
		rgb.value()
	} else {
		config::Rgb(0, 0, 0)
	}
}

fn key_to_item(item: Option<&mut Item>, value: config::Keycode) {
	if let Some(Item::Input(_, Input::Key(key))) = item {
		key.set_value(*value);
	}
}

fn key_from_item(item: Option<&Item>) -> config::Keycode {
	if let Some(Item::Input(_, Input::Key(k))) = item {
		k.value().into()
	} else {
		sdl2::keyboard::Keycode::Power.into()
	}
}

fn usize_from_item(item: Option<&Item>) -> usize {
	if let Some(Item::Input(_, Input::Int(i))) = item {
		return i.value();
	}
	0
}

fn usize_to_label(item: Option<&mut Item>, n: usize) {
	match item {
		Some(Item::Label2(_, value)) | Some(Item::Label(value, _)) | Some(Item::Text(value)) => {
			value.clear();
			let _ = write!(value, "{}", n);
		}
		_ => {}
	}
}

fn str_to_label<S: AsRef<str>>(item: Option<&mut Item>, s: S) {
	match item {
		Some(Item::Label2(_, value)) | Some(Item::Label(value, _)) | Some(Item::Text(value)) => {
			value.clear();
			let _ = write!(value, "{}", s.as_ref());
		}
		_ => {}
	}
}
