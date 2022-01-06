use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
	pub fn from_tuple(rgb: (u8, u8, u8)) -> Self {
		Rgb(rgb.0, rgb.1, rgb.2)
	}

	pub fn rgb(&self) -> (u8, u8, u8) {
		(self.0, self.1, self.2)
	}
}

impl Serialize for Rgb {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2))
	}
}

impl<'de> Deserialize<'de> for Rgb {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		if s.starts_with('#') && s.len() == 7 {
			return Ok(Rgb(
				u8::from_str_radix(&s[1..3], 16).map_err(|_| {
					de::Error::custom(&format!("Invalid R component: {}", &s[1..3]))
				})?,
				u8::from_str_radix(&s[3..5], 16).map_err(|_| {
					de::Error::custom(&format!("Invalid G component: {}", &s[3..5]))
				})?,
				u8::from_str_radix(&s[5..7], 16).map_err(|_| {
					de::Error::custom(&format!("Invalid B component: {}", &s[5..7]))
				})?,
			));
		}
		Err(de::Error::custom(&format!("Invalid RGB: {}", s)))
	}
}
