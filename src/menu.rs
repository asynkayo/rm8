use crate::{
	m8::M8,
	config::{Command, Config, DEFAULT_SENSIBILITY},
	nav::{Navigation, Page},
};

pub fn build_menu(menu: &mut Navigation, m8: &M8, config: &Config) {
	let mut theme_page = Page::new("THEME", 'T');
	theme_page.add_rgb("TEXT:DEFAULT", config.theme.text_default);
	theme_page.add_rgb("TEXT:VALUE", config.theme.text_value);
	theme_page.add_rgb("TEXT:TITLE", config.theme.text_title);
	theme_page.add_rgb("TEXT:INFO", config.theme.text_info);
	theme_page.add_rgb("CURSOR", config.theme.cursor);
	theme_page.add_rgb("SCREEN", config.theme.screen);
	theme_page.add_rgb("VELOCITY FG", config.theme.velocity_fg);
	theme_page.add_rgb("VELOCITY BG", config.theme.velocity_bg);
	theme_page.add_rgb("OCTAVE FG", config.theme.octave_fg);
	theme_page.add_rgb("OCTAVE BG", config.theme.octave_bg);
	theme_page.add_empty();
	theme_page.add_action2("RESET", "SAVE");

	let mut app_page = Page::new("CONFIG", 'C');
	app_page.add_bool("FULLSCREEN", config.app.fullscreen);
	app_page.add_int("ZOOM", config.app.zoom as usize, 1, 9, 2);
	app_page.add_font("FONT", config.app.font);
	app_page.add_int("KEY SENS.", config.app.key_sensibility as usize, 60, 200, 10);
	app_page.add_bool("SHOW_FPS", config.app.show_fps);
	app_page.add_int("FPS", config.app.fps, 1, 200, 10);
	app_page.add_bool("RECONNECT", config.app.reconnect);
	app_page.add_device("DEVICE", m8.device_name());
	app_page.add_empty();
	app_page.add_action2("RESET", "SAVE");
	app_page.add_page_above(theme_page);

	let mut rm8key_page = Page::new("RM8 KEYS", 'R');
	rm8key_page.add_key("KEYJAZZ", *config.rm8.keyjazz);
	rm8key_page.add_key("VELOCITY-", *config.rm8.velocity_minus);
	rm8key_page.add_key("VELOCITY+", *config.rm8.velocity_plus);
	rm8key_page.add_key("OCTAVE-", *config.rm8.octave_minus);
	rm8key_page.add_key("OCTAVE+", *config.rm8.octave_plus);
	rm8key_page.add_empty();
	rm8key_page.add_action3("REMAP", "RESET", "SAVE");

	let mut m8key_page = Page::new("M8 KEYS", 'K');
	m8key_page.add_key("UP", *config.m8.up);
	m8key_page.add_key("DOWN", *config.m8.down);
	m8key_page.add_key("LEFT", *config.m8.left);
	m8key_page.add_key("RIGHT", *config.m8.right);
	m8key_page.add_key("EDIT", *config.m8.edit);
	m8key_page.add_key("OPTION", *config.m8.option);
	m8key_page.add_key("SHIFT", *config.m8.shift);
	m8key_page.add_key("PLAY", *config.m8.play);
	m8key_page.add_empty();
	m8key_page.add_action3("REMAP", "RESET", "SAVE");
	m8key_page.add_page_below(rm8key_page);

	let mut empty_joystick_page = Page::new("JOYSTICK", 'J');
	empty_joystick_page.add_info("N.JOYSTICKS", "0");

	menu.add_page(app_page);
	menu.add_page(m8key_page);
	menu.add_page(empty_joystick_page);
}

pub fn build_joystick_page() -> Option<Page> {
	let mut axes_page = Page::new("AXES", 'A');
	axes_page.add_title2("NEG.", "POS.", Command::MAX_LENGTH);
	axes_page.add_cmd2("AXIS 0", Command::None, Command::None);
	axes_page.add_int("AXIS 0 SENS.", DEFAULT_SENSIBILITY, 0, i16::MAX as usize, 100);
	axes_page.add_cmd2("AXIS 1", Command::None, Command::None);
	axes_page.add_int("AXIS 1 SENS.", DEFAULT_SENSIBILITY, 0, i16::MAX as usize, 100);
	axes_page.add_cmd2("AXIS 2", Command::None, Command::None);
	axes_page.add_int("AXIS 2 SENS.", DEFAULT_SENSIBILITY, 0, i16::MAX as usize, 100);
	axes_page.add_cmd2("AXIS 3", Command::None, Command::None);
	axes_page.add_int("AXIS 3 SENS.", DEFAULT_SENSIBILITY, 0, i16::MAX as usize, 100);
	axes_page.add_cmd2("AXIS 4", Command::None, Command::None);
	axes_page.add_int("AXIS 4 SENS.", DEFAULT_SENSIBILITY, 0, i16::MAX as usize, 100);
	axes_page.add_cmd2("AXIS 5", Command::None, Command::None);
	axes_page.add_int("AXIS 5 SENS.", DEFAULT_SENSIBILITY, 0, i16::MAX as usize, 100);
	axes_page.add_empty();
	axes_page.add_action2("RESET", "SAVE");

	let mut buttons_page = Page::new("BUTTONS", 'B');
	buttons_page.add_cmd_label2("B.0", Command::None, "B.10", Command::None, 4);
	buttons_page.add_cmd_label2("B.1", Command::None, "B.11", Command::None, 4);
	buttons_page.add_cmd_label2("B.2", Command::None, "B.12", Command::None, 4);
	buttons_page.add_cmd_label2("B.3", Command::None, "B.13", Command::None, 4);
	buttons_page.add_cmd_label2("B.4", Command::None, "B.14", Command::None, 4);
	buttons_page.add_cmd_label2("B.5", Command::None, "B.15", Command::None, 4);
	buttons_page.add_cmd_label2("B.6", Command::None, "B.16", Command::None, 4);
	buttons_page.add_cmd_label2("B.7", Command::None, "B.17", Command::None, 4);
	buttons_page.add_cmd_label2("B.8", Command::None, "B.18", Command::None, 4);
	buttons_page.add_cmd_label2("B.9", Command::None, "B.19", Command::None, 4);
	buttons_page.add_empty();
	buttons_page.add_action2("RESET", "SAVE");

	let mut hats_page = Page::new("HAT", 'H');
	hats_page.add_cmd("UP", Command::None);
	hats_page.add_cmd("DOWN", Command::None);
	hats_page.add_cmd("LEFT", Command::None);
	hats_page.add_cmd("RIGHT", Command::None);
	hats_page.add_cmd("UP LEFT", Command::None);
	hats_page.add_cmd("UP RIGHT", Command::None);
	hats_page.add_cmd("DOWN LEFT", Command::None);
	hats_page.add_cmd("DOWN RIGHT", Command::None);
	hats_page.add_empty();
	hats_page.add_action2("RESET", "SAVE");

	let mut joystick_page = Page::new("JOYSTICK", 'J');
	joystick_page.add_info("N.JOYSTICKS", "0");
	joystick_page.add_int("SEL.ID", 0, 0, 10, 1);
	joystick_page.add_empty();
	joystick_page.add_title("");
	joystick_page.add_text("GUID");
	joystick_page.add_info("N.AXES", "0");
	joystick_page.add_info("N.BUTTONS", "0");
	joystick_page.add_info("N.HATS", "0");
	joystick_page.add_empty();
	joystick_page.add_action2("RESET", "SAVE");
	joystick_page.add_page_above(axes_page);
	joystick_page.add_page_above(buttons_page);
	joystick_page.add_page_below(hats_page);
	Some(joystick_page)
}
