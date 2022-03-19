use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

use std::sync::mpsc;

struct Capture {
	done_sender: mpsc::Sender<Vec<i16>>,
	size: usize,
}

impl AudioCallback for Capture {
	type Channel = i16;

	fn callback(&mut self, input: &mut [i16]) {
		for chunk in input.chunks(self.size) {
			self.done_sender.send(chunk.to_owned()).expect("could not send record buffer");
		}
	}
}

struct Playback {
	done_receiver: mpsc::Receiver<Vec<i16>>,
}

impl AudioCallback for Playback {
	type Channel = i16;

	fn callback(&mut self, output: &mut [i16]) {
		if let Ok(vec) = self.done_receiver.recv() {
			output.copy_from_slice(&vec);
		}
	}
}

pub struct Audio {
	capture: AudioDevice<Capture>,
	playback: AudioDevice<Playback>,
	playing: bool,
	name: String,
}

impl Audio {
	fn real_open(audio: &sdl2::AudioSubsystem, device_name: String) -> Result<Self, String> {
		let spec = AudioSpecDesired { freq: Some(44100), channels: None, samples: None };
		let (done_sender, done_receiver) = mpsc::channel();
		let capture = audio.open_capture(Some(device_name.as_ref()), &spec, |spec| Capture {
			done_sender,
			size: spec.samples as usize * 2,
		})?;
		let playback = audio.open_playback(None, &spec, |_spec| Playback { done_receiver })?;

		return Ok(Self { playing: false, capture, playback, name: device_name });
	}

	pub fn open(audio: &sdl2::AudioSubsystem, name: Option<String>) -> Result<Self, String> {
		if let Some(device_name) = name {
			Self::real_open(audio, device_name)
		} else {
			for i in 0..audio.num_audio_capture_devices().unwrap_or(0) {
				if let Ok(device_name) = audio.audio_capture_device_name(i) {
					if device_name.starts_with("M8 Analog Stereo") {
						return Self::real_open(audio, device_name);
					}
				}
			}
			Err("No M8 audio device found".to_string())
		}
	}

	pub fn reopen(&mut self, audio: &sdl2::AudioSubsystem, name: String) -> Result<(), String> {
		*self = Self::real_open(audio, name)?;
		Ok(())
	}

	pub fn toggle(&mut self) {
		if self.playing {
			self.pause()
		} else {
			self.resume()
		}
	}

	pub fn pause(&mut self) {
		self.playing = false;
		self.capture.pause();
		self.playback.pause();
	}

	pub fn resume(&mut self) {
		self.playing = true;
		self.capture.resume();
		self.playback.resume();
	}

	pub fn name(&self) -> String {
		self.name.clone()
	}
}
