use std::env;

use crate::config::Config;
use crate::m8::M8;

const USAGE: &str = "Usage rm8 [options]
Available options:
	-help		 Display this help screen
	-version	 Display the version of the program
	-list		 List available M8 devices
	-noaudio	 Disable audio loopback mode
	-dev DEVICE	 Connect to the the given M8 device
	-cap DEVICE  Connect the given capture device to the default playback device
	-smp SAMPLES Use the specified number of samples for audio processing
	-wc			 Write the default configuration to the standard output
	-wc FILE	 Write the default configuration to the given file
	-rc FILE	 Read the configuration from the given file";

pub fn handle_command_line(
	config: &mut Config,
	config_file: &mut Option<String>,
	device: &mut Option<String>,
	capture: &mut Option<String>,
	samples: &mut Option<u16>,
	noaudio: &mut bool,
) -> Result<bool, String> {
	let mut args = env::args().skip(1);
	loop {
		match args.next().as_deref() {
			Some("-version") => {
				println!("rm8 v{}", env!("CARGO_PKG_VERSION"));
				return Ok(false);
			}
			Some("-help") => {
				eprintln!("{}", USAGE);
				return Ok(false);
			}
			Some("-list") => {
				let ports = M8::list_ports().map_err(|e| e.to_string())?;
				println!("{}", if ports.is_empty() { "No M8 found" } else { "M8 found:" });
				for port in ports {
					println!("\t- {}", port);
				}
				return Ok(false);
			}
			Some("-wc") => match args.next() {
				Some(file) => {
					if let Err(e) = config.write(&file) {
						return Err(format!("Error: writing config to file {} ({})", &file, e));
					}
					config_file.replace(file);
					return Ok(false);
				}
				None => match config.dump() {
					Ok(json) => {
						println!("{}", json);
						return Ok(false);
					}
					Err(e) => return Err(format!("Error: dumping config ({})", e)),
				},
			},
			Some("-rc") => match args.next() {
				Some(file) => {
					if let Err(e) = config.read(&file) {
						return Err(format!("Error: loading config file `{}` ({})", file, e));
					}
					config_file.replace(file);
				}
				None => return Err("Error: missing config file argument".to_string()),
			},
			Some("-noaudio") => {
				*noaudio = true;
			}
			Some("-dev") => match args.next() {
				Some(dev) => {
					device.replace(dev);
				}
				None => return Err("Error: missing device argument".to_string()),
			},
			Some("-cap") => match args.next() {
				Some(cap) => {
					capture.replace(cap);
				}
				None => return Err("Error: missing capture argument".to_string()),
			},
			Some("-smp") => match args.next() {
				Some(smp) => {
					let smp =
						smp.parse().map_err(|_| "Error: invalid samples argument".to_string())?;
					samples.replace(smp);
				}
				None => return Err("Error: missing samples argument".to_string()),
			},
			Some(arg) => return Err(format!("Error: unknown argument: {}", arg)),
			None => break,
		}
	}
	Ok(true)
}
