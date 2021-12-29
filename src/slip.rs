use std::io;

use serialport::SerialPort;

const END: u8 = 0xc0;
const ESC: u8 = 0xdb;
const ESC_END: u8 = 0xdc;
const ESC_ESC: u8 = 0xdd;

enum SlipState {
	Normal,
	Escape,
}

pub struct Slip<const N: usize> {
	state: SlipState,
	buf: [u8; N],
	rmax: usize,
	rpos: usize,
	wpos: usize,
}

impl<const N: usize> Slip<N> {
	pub fn new() -> Self {
		Self { state: SlipState::Normal, buf: [0; N], rmax: 0, rpos: 0, wpos: 0 }
	}

	fn push_byte(&mut self, byte: u8, buf: &mut [u8]) -> Result<(), String> {
		if self.wpos >= buf.len() {
			self.wpos = 0;
			return Err("push_byte overflow".to_string());
		}
		buf[self.wpos] = byte;
		self.wpos += 1;
		Ok(())
	}

	pub fn read<'a>(
		&mut self,
		port: &mut Box<dyn SerialPort>,
		buf: &'a mut [u8],
	) -> Result<Option<&'a [u8]>, String> {
		loop {
			if self.rpos >= self.rmax {
				self.rpos = 0;
				match port.read(&mut self.buf) {
					Ok(n) => self.rmax = n,
					Err(e) if e.kind() == io::ErrorKind::TimedOut => self.rmax = 0,
					Err(e) => return Err(e.to_string()),
				}
				if self.rmax == 0 {
					return Ok(None);
				}
			}
			while self.rpos < self.rmax {
				let byte = self.buf[self.rpos];
				match self.state {
					SlipState::Normal => match byte {
						END if self.wpos > 1 => {
							self.rpos += 1;
							let end = self.wpos;
							self.wpos = 0;
							return Ok(Some(&buf[..end]));
						}
						END => return Err("empty command".to_string()),
						ESC => {
							self.state = SlipState::Escape;
							self.rpos += 1;
							continue;
						}
						_ => self.push_byte(byte, buf)?,
					},
					SlipState::Escape => match byte {
						ESC_END => self.push_byte(END, buf)?,
						ESC_ESC => self.push_byte(ESC, buf)?,
						_ => return Err(format!("invalid escape sequence: {:02x}", byte)),
					},
				}
				self.state = SlipState::Normal;
				self.rpos += 1;
			}
		}
	}
}
