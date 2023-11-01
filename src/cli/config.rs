use crate::{confirm::prompt, unwrap_or_return};
use clap::Parser;
use directories::UserDirs;
use log::{error, trace};
use open;
use std::{fs::File, path::Path};

/// Edit global config with default editor
#[derive(Parser)]
pub struct Command {}

impl Command {
	pub fn run(self) {
		let user_dirs = unwrap_or_return!(UserDirs::new());
		let home_dir = user_dirs.home_dir();
		let config_dir = home_dir.join(Path::new(".argon/config.toml"));

		if !config_dir.exists() {
			let create_config = prompt("Config does not exist. Would you like to create one?", true);

			if create_config.unwrap_or(false) {
				match File::create(&config_dir) {
					Err(error) => {
						error!("Failed to create config file: {error}");
						return;
					}
					Ok(_) => trace!("Created config file"),
				}
			} else {
				return;
			}
		}

		match open::that(config_dir) {
			Err(error) => error!("Failed to open config file: {error}"),
			Ok(()) => trace!("Opening config file"),
		}
	}
}
