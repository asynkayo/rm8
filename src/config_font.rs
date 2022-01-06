use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Font {
	Uppercase,
	UpperAltZero,
	Lowercase,
	LowerAltZero,
}

impl Font {
	pub const MAX_LENGTH: usize = 14;
}

impl TryFrom<u8> for Font {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		Ok(match value {
			0 => Font::Uppercase,
			1 => Font::UpperAltZero,
			2 => Font::Lowercase,
			3 => Font::LowerAltZero,
			_ => return Err(()),
		})
	}
}
impl fmt::Display for Font {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Font::Uppercase => write!(f, "UPPERCASE"),
			Font::UpperAltZero => write!(f, "UPPER ALT.ZERO"),
			Font::Lowercase => write!(f, "LOWERCASE"),
			Font::LowerAltZero => write!(f, "LOWER ALT.ZERO"),
		}
	}
}
