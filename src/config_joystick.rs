use serde::{ser::SerializeMap, Serializer};
use std::collections::HashMap;

use crate::config::Command;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HatConfig {
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub up: Command,
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub down: Command,
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub left: Command,
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub right: Command,
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub left_up: Command,
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub right_up: Command,
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub left_down: Command,
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub right_down: Command,
}

pub const DEFAULT_SENSIBILITY: usize = 20000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axis {
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub negative: Command,
	#[serde(default, skip_serializing_if = "Command::is_none")]
	pub positive: Command,
	#[serde(default)]
	pub sensibility: usize,
}

impl Axis {
	pub fn new(negative: Command, positive: Command, sensibility: usize) -> Self {
		Self { negative, positive, sensibility }
	}
}

impl Default for Axis {
	fn default() -> Self {
		Self { negative: Command::None, positive: Command::None, sensibility: DEFAULT_SENSIBILITY }
	}
}

fn serialize_buttons<S: Serializer>(
	buttons: &HashMap<u8, Command>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	let mut map = serializer.serialize_map(None)?;
	for (k, v) in buttons.iter() {
		if v != &Command::None {
			map.serialize_entry(k, v)?;
		}
	}
	map.end()
}

fn serialize_axes<S: Serializer>(
	axes: &HashMap<u8, Axis>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	let mut map = serializer.serialize_map(None)?;
	for (k, v) in axes.iter() {
		if v.negative != Command::None || v.positive != Command::None || v.sensibility != 0 {
			map.serialize_entry(k, v)?;
		}
	}
	map.end()
}

fn buttons_empty(buttons: &HashMap<u8, Command>) -> bool {
	for (_, v) in buttons.iter() {
		if v != &Command::None {
			return false;
		}
	}
	true
}

fn axes_empty(axes: &HashMap<u8, Axis>) -> bool {
	for (_, v) in axes.iter() {
		if v.negative != Command::None || v.positive != Command::None || v.sensibility != 0 {
			return false;
		}
	}
	true
}

pub fn serialize_joysticks<S: Serializer>(
	joysticks: &HashMap<String, JoystickConfig>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	let mut map = serializer.serialize_map(None)?;
	for (k, v) in joysticks.iter() {
		if v.hats.is_some() || !buttons_empty(&v.buttons) || !axes_empty(&v.axes) {
			map.serialize_entry(k, v)?;
		}
	}
	map.end()
}

pub fn joysticks_empty(joystick: &HashMap<String, JoystickConfig>) -> bool {
	for (_, v) in joystick.iter() {
		if v.hats.is_some() || !buttons_empty(&v.buttons) || !axes_empty(&v.axes) {
			return false;
		}
	}
	true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JoystickConfig {
	#[serde(default, skip_serializing_if = "buttons_empty", serialize_with = "serialize_buttons")]
	pub buttons: HashMap<u8, Command>,
	#[serde(default, skip_serializing_if = "axes_empty", serialize_with = "serialize_axes")]
	pub axes: HashMap<u8, Axis>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hats: Option<HashMap<u32, HatConfig>>,
}
