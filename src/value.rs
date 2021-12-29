pub struct Value<T> {
	value: T,
	modified: bool,
}

impl<T> std::ops::Deref for Value<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T: std::cmp::Eq + Copy> Value<T> {
	pub fn new(value: T) -> Self {
		Self { value, modified: false }
	}

	pub fn set(&mut self, value: T) {
		if self.value != value {
			self.value = value;
			self.modified = true;
		}
	}

	pub fn changed(&mut self) -> bool {
		if self.modified {
			self.modified = false;
			return true;
		}
		false
	}
}

impl Value<bool> {
	pub fn toggle(&mut self) {
		self.value = !self.value;
		self.modified = true;
	}
}

impl Value<u8> {
	pub fn set_bit(&mut self, mask: u8) {
		if self.value & mask == 0 {
			self.value |= mask;
			self.modified = true;
		}
	}

	pub fn clr_bit(&mut self, mask: u8) {
		if self.value & mask != 0 {
			self.value &= !mask;
			self.modified = true;
		}
	}

	pub fn add(&mut self, add: u8, max: u8) {
		if self.value < max {
			self.modified = true;
			if self.value as usize + add as usize > max as usize {
				self.value = max;
			} else {
				self.value += add;
			}
		}
	}

	pub fn sub(&mut self, sub: u8, min: u8) {
		if self.value > min {
			self.modified = true;
			if self.value as usize > min as usize + sub as usize {
				self.value -= sub;
			} else {
				self.value = min;
			}
		}
	}
}
