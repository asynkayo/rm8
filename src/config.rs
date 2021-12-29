use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sdl2::keyboard::{Scancode};

use crate::m8;

pub const FILE: &str = "rm8.json";

pub struct M8Key(Scancode);

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

#[derive(Serialize, Deserialize)]
pub struct Config {
	pub up: M8Key,
	pub down: M8Key,
	pub left: M8Key,
	pub right: M8Key,
	pub shift: M8Key,
	pub play: M8Key,
	pub edit: M8Key,
	pub option: M8Key,
	pub keyjazz: M8Key,
	pub octave_plus: M8Key,
	pub octave_minus: M8Key,
	pub velocity_plus: M8Key,
	pub velocity_minus: M8Key,
	pub fullscreen: bool,
	#[serde(skip)]
	pub overlap: bool,
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

pub const KEYJAZZ: [Scancode; 34] = [
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

	pub fn handle_keys(&self, m8: &mut m8::M8, code: Scancode, on: bool) {
		let mut mask: u8 = 0;
		if code == *self.up {
			mask |= m8::KEYS_UP;
		} else if code == *self.down {
			mask |= m8::KEYS_DOWN;
		} else if code == *self.left {
			mask |= m8::KEYS_LEFT;
		} else if code == *self.right {
			mask |= m8::KEYS_RIGHT;
		} else if code == *self.shift {
			mask |= m8::KEYS_SHIFT;
		} else if code == *self.play {
			mask |= m8::KEYS_PLAY;
		} else if code == *self.edit {
			mask |= m8::KEYS_EDIT;
		} else if code == *self.option {
			mask |= m8::KEYS_OPTION;
		}
		if on {
			m8.keys.set_bit(mask);
		} else {
			m8.keys.clr_bit(mask);
		}
	}

	pub fn handle_keyjazz(&mut self, m8: &mut m8::M8, code: Scancode, fast: bool) {
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

pub struct Remap {
	mapping: [Scancode; 13],
	item: usize,
	pub exists: bool,
	pub init: bool,
}

impl Remap {
	pub fn new() -> Self {
		Self { mapping: [Scancode::A; 13], item: 0, exists: false, init: true }
	}

	pub fn map(&mut self, key: Scancode) {
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

	pub fn done(&self) -> bool {
		self.item >= self.mapping.len()
	}

	pub fn current(&self) -> &'static str {
		KEY_NAMES[self.item]
	}

	pub fn write(&self, config: &mut Config) {
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

