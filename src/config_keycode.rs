use sdl2::keyboard::Keycode as SdlKeycode;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keycode(pub SdlKeycode);

impl std::ops::Deref for Keycode {
	type Target = SdlKeycode;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<SdlKeycode> for Keycode {
	fn from(keycode: SdlKeycode) -> Self {
		Self(keycode)
	}
}

impl Serialize for Keycode {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.0.name())
	}
}

impl<'de> Deserialize<'de> for Keycode {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		match SdlKeycode::from_name(&s) {
			Some(code) => Ok(Self(code)),
			None => Err(de::Error::custom(&format!("Invalid key: {}", s))),
		}
	}
}
