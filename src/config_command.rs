use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Command {
	None,
	Up,
	Down,
	Left,
	Right,
	Edit,
	r#Option,
	Shift,
	Play,
	VelocityMinus,
	VelocityPlus,
	OctaveMinus,
	OctavePlus,
	Keyjazz,
}

impl Default for Command {
	fn default() -> Self {
		Self::None
	}
}

impl Command {
	pub const MAX_LENGTH: usize = 9;

	pub fn is_none(&self) -> bool {
		self == &Self::None
	}
}

impl TryFrom<u8> for Command {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		Ok(match value {
			0 => Command::None,
			1 => Command::Up,
			2 => Command::Down,
			3 => Command::Left,
			4 => Command::Right,
			5 => Command::Edit,
			6 => Command::r#Option,
			7 => Command::Shift,
			8 => Command::Play,
			9 => Command::VelocityMinus,
			10 => Command::VelocityPlus,
			11 => Command::OctaveMinus,
			12 => Command::OctavePlus,
			13 => Command::Keyjazz,
			_ => return Err(()),
		})
	}
}

impl fmt::Display for Command {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			Command::None => write!(f, "{:1$}", " ", Command::MAX_LENGTH),
			Command::Up => write!(f, "UP"),
			Command::Down => write!(f, "DOWN"),
			Command::Left => write!(f, "LEFT"),
			Command::Right => write!(f, "RIGHT"),
			Command::Edit => write!(f, "EDIT"),
			Command::r#Option => write!(f, "OPTION"),
			Command::Shift => write!(f, "SHIFT"),
			Command::Play => write!(f, "PLAY"),
			Command::VelocityMinus => write!(f, "VELOCITY-"),
			Command::VelocityPlus => write!(f, "VELOCITY+"),
			Command::OctaveMinus => write!(f, "OCTAVE-"),
			Command::OctavePlus => write!(f, "OCTAVE+"),
			Command::Keyjazz => write!(f, "KEYJAZZ"),
		}
	}
}
