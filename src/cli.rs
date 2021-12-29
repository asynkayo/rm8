use std::env;

pub enum Action {
	Continue,
	Return,
}

use crate::m8::M8;
use crate::config::Config;

const USAGE: &str = "Usage rm8 [options]
Available options:
	-help		Display this help screen
	-version	Display the version of the program
	-list		List available M8 devices
	-dev DEVICE	Connect to the the given M8 device
	-wc		Write the default configuration to the standard output
	-wc FILE	Write the default configuration to the given file
	-rc FILE	Read the configuration from the given file";

pub fn handle_command_line(
	config: &mut Config,
	config_file: &mut Option<String>,
	device: &mut Option<String>,
) -> Result<Action, String> {
	let mut args = env::args().skip(1);
	match (args.next().as_deref(), args.next()) {
		(Some("-version"), None) => println!("rm8 v{}", env!("CARGO_PKG_VERSION")),
		(Some("-help"), None) => eprintln!("{}", USAGE),
		(Some("-list"), None) => {
			let ports = M8::list_ports().map_err(|e| e.to_string())?;
			println!("{}", if ports.is_empty() { "No M8 found" } else { "M8 found:" });
			for port in ports {
				println!("\t- {}", port);
			}
		}
		(Some("-wc"), Some(file)) => {
			if let Err(e) = config.write(&file) {
				return Err(format!("Error: writing config to file {} ({})", &file, e));
			}
			config_file.replace(file);
		}
		(Some("-wc"), None) => match config.dump() {
			Ok(json) => println!("{}", json),
			Err(e) => return Err(format!("Error: dumping config ({})", e)),
		},
		(Some("-rc"), Some(file)) => {
			if let Err(e) = config.read(&file) {
				return Err(format!("Error: loading config file `{}` ({})", file, e));
			}
			return Ok(Action::Continue);
		}
		(Some("-rc"), None) => return Err("Error: missing config file argument".to_string()),
		(Some("-dev"), Some(dev)) => {
			device.replace(dev);
			return Ok(Action::Continue);
		}
		(Some("-dev"), None) => return Err("Error: missing device argument".to_string()),
		_ => return Ok(Action::Continue),
	};
	Ok(Action::Return)
}
