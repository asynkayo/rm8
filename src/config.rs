use sdl2::keyboard::Keycode as SdlKeycode;
use std::collections::HashMap;

pub use crate::config_command::Command;
pub use crate::config_font::Font;
use crate::config_joystick::{joysticks_empty, serialize_joysticks};
pub use crate::config_joystick::{Axis, HatConfig, JoystickConfig, DEFAULT_SENSIBILITY};
pub use crate::config_keycode::Keycode;
pub use crate::config_rgb::Rgb;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
	pub screen: Rgb,
	pub text_default: Rgb,
	pub text_value: Rgb,
	pub text_title: Rgb,
	pub text_info: Rgb,
	pub cursor: Rgb,
	pub octave_bg: Rgb,
	pub octave_fg: Rgb,
	pub velocity_bg: Rgb,
	pub velocity_fg: Rgb,
}

impl Default for ThemeConfig {
	fn default() -> Self {
		Self {
			octave_bg: Rgb(0, 0, 255),
			octave_fg: Rgb(255, 255, 255),
			velocity_bg: Rgb(255, 0, 0),
			velocity_fg: Rgb(255, 255, 255),
			screen: Rgb(0, 0, 0),
			text_default: Rgb(0x8c, 0x8c, 0xba),
			text_value: Rgb(0xfa, 0xfa, 0xfa),
			text_title: Rgb(0x32, 0xec, 0xff),
			text_info: Rgb(0x60, 0x60, 0x8e),
			cursor: Rgb(0x32, 0xec, 0xff),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
	pub fullscreen: bool,
	pub font: Font,
	pub zoom: u32,
	pub key_sensibility: u64,
	pub fps: usize,
	pub show_fps: bool,
	pub reconnect: bool,
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			fullscreen: false,
			font: Font::Uppercase,
			zoom: 4,
			key_sensibility: 60,
			fps: 60,
			show_fps: false,
			reconnect: false,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct M8KeyboardConfig {
	pub up: Keycode,
	pub down: Keycode,
	pub left: Keycode,
	pub right: Keycode,
	pub edit: Keycode,
	pub option: Keycode,
	pub shift: Keycode,
	pub play: Keycode,
}

impl Default for M8KeyboardConfig {
	fn default() -> Self {
		Self {
			up: SdlKeycode::Up.into(),
			down: SdlKeycode::Down.into(),
			left: SdlKeycode::Left.into(),
			right: SdlKeycode::Right.into(),
			edit: SdlKeycode::LCtrl.into(),
			option: SdlKeycode::LAlt.into(),
			shift: SdlKeycode::LShift.into(),
			play: SdlKeycode::Space.into(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RM8KeyboardConfig {
	pub keyjazz: Keycode,
	pub velocity_minus: Keycode,
	pub velocity_plus: Keycode,
	pub octave_minus: Keycode,
	pub octave_plus: Keycode,
}

impl Default for RM8KeyboardConfig {
	fn default() -> Self {
		Self {
			keyjazz: SdlKeycode::Return.into(),
			velocity_minus: SdlKeycode::Minus.into(),
			velocity_plus: SdlKeycode::Equals.into(),
			octave_minus: SdlKeycode::LeftBracket.into(),
			octave_plus: SdlKeycode::RightBracket.into(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub app: AppConfig,
	pub theme: ThemeConfig,
	pub m8: M8KeyboardConfig,
	pub rm8: RM8KeyboardConfig,
	#[serde(
		default,
		skip_serializing_if = "joysticks_empty",
		serialize_with = "serialize_joysticks"
	)]
	pub joysticks: HashMap<String, JoystickConfig>,
	pub keyjazz: HashMap<Keycode, u8>,
	#[serde(skip)]
	pub overlap: bool,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			app: AppConfig::default(),
			theme: ThemeConfig::default(),
			m8: M8KeyboardConfig::default(),
			rm8: RM8KeyboardConfig::default(),
			keyjazz: HashMap::from([
				(Keycode(SdlKeycode::Z), 0),
				(Keycode(SdlKeycode::S), 1),
				(Keycode(SdlKeycode::X), 2),
				(Keycode(SdlKeycode::D), 3),
				(Keycode(SdlKeycode::C), 4),
				(Keycode(SdlKeycode::V), 5),
				(Keycode(SdlKeycode::G), 6),
				(Keycode(SdlKeycode::B), 7),
				(Keycode(SdlKeycode::H), 8),
				(Keycode(SdlKeycode::N), 9),
				(Keycode(SdlKeycode::J), 10),
				(Keycode(SdlKeycode::M), 11),
				(Keycode(SdlKeycode::Comma), 12),
				(Keycode(SdlKeycode::L), 13),
				(Keycode(SdlKeycode::Period), 14),
				(Keycode(SdlKeycode::Semicolon), 15),
				(Keycode(SdlKeycode::Slash), 16),
				(Keycode(SdlKeycode::Q), 12),
				(Keycode(SdlKeycode::Num2), 13),
				(Keycode(SdlKeycode::W), 14),
				(Keycode(SdlKeycode::Num3), 15),
				(Keycode(SdlKeycode::E), 16),
				(Keycode(SdlKeycode::R), 17),
				(Keycode(SdlKeycode::Num5), 18),
				(Keycode(SdlKeycode::T), 19),
				(Keycode(SdlKeycode::Num6), 20),
				(Keycode(SdlKeycode::Y), 21),
				(Keycode(SdlKeycode::Num7), 22),
				(Keycode(SdlKeycode::U), 23),
				(Keycode(SdlKeycode::I), 24),
				(Keycode(SdlKeycode::Num9), 25),
				(Keycode(SdlKeycode::O), 26),
				(Keycode(SdlKeycode::Num0), 27),
				(Keycode(SdlKeycode::P), 28),
			]),
			joysticks: HashMap::new(),
			overlap: false,
		}
	}
}

impl Config {
	pub fn read<T: AsRef<str>>(&mut self, file: T) -> Result<(), String> {
		let content = std::fs::read_to_string(file.as_ref()).map_err(|e| e.to_string())?;
		let config: Self = serde_json::from_str(&content).map_err(|e| e.to_string())?;
		*self = config;
		self.check_overlap();
		Ok(())
	}

	pub fn write<T: AsRef<str>>(&self, file: T) -> Result<(), String> {
		let config = self.dump()?;
		std::fs::write(file.as_ref(), config).map_err(|e| e.to_string())
	}

	pub fn dump(&self) -> Result<String, String> {
		serde_json::to_string_pretty(self).map_err(|e| e.to_string())
	}

	fn check_overlap(&mut self) {
		self.overlap = self.keyjazz.contains_key(&self.m8.up)
			|| self.keyjazz.contains_key(&self.m8.down)
			|| self.keyjazz.contains_key(&self.m8.left)
			|| self.keyjazz.contains_key(&self.m8.right)
			|| self.keyjazz.contains_key(&self.m8.shift)
			|| self.keyjazz.contains_key(&self.m8.play)
			|| self.keyjazz.contains_key(&self.m8.edit)
			|| self.keyjazz.contains_key(&self.m8.option)
			|| self.keyjazz.contains_key(&self.rm8.keyjazz)
			|| self.keyjazz.contains_key(&self.rm8.octave_minus)
			|| self.keyjazz.contains_key(&self.rm8.octave_plus)
			|| self.keyjazz.contains_key(&self.rm8.velocity_minus)
			|| self.keyjazz.contains_key(&self.rm8.velocity_plus);
	}
}
