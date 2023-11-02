use crate::{argon_error, confirm::prompt, utils::get_home_dir};
use clap::Parser;
use log::trace;
use open;
use std::{fs::File, path::Path};

/// Edit global config with default editor
#[derive(Parser)]
pub struct Command {}

impl Command {
	pub fn run(self) {
		let home_dir = get_home_dir();

		match home_dir {
			Err(error) => {
				argon_error!("Failed to locate config: {}", error);
				return;
			}
			Ok(_) => trace!("Retrieved home dir"),
		}

		let config_dir = home_dir.unwrap().join(Path::new(".argon/config.toml"));

		if !config_dir.exists() {
			let create_config = prompt("Config does not exist. Would you like to create one?", true);

			if create_config.unwrap_or(false) {
				match File::create(&config_dir) {
					Err(error) => {
						argon_error!("Failed to create config file: {error}");
						return;
					}
					Ok(_) => trace!("Created config file"),
				}
			} else {
				return;
			}
		}

		match open::that(config_dir) {
			Err(error) => argon_error!("Failed to open config file: {error}"),
			Ok(()) => trace!("Opening config file"),
		}
	}
}
