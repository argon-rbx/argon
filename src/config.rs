use directories::UserDirs;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::Path};
use toml;

use crate::unwrap_or_return;

macro_rules! set_if_some {
	($default:expr, $optional:expr) => {
		if $optional.is_some() {
			$default = $optional.unwrap();
		}
	};
}

#[derive(Serialize, Deserialize)]
struct GlobalConfig {
	host: Option<String>,
	port: Option<u16>,
}

// TODO
#[derive(Serialize, Deserialize)]
struct ProjectConfig {
	host: Option<String>,
	port: Option<u16>,
}

pub struct Config {
	pub host: String,
	pub port: u16,
}

impl Config {
	pub fn new() -> Config {
		let mut config = Config {
			host: String::from("localhost"),
			port: 8000,
		};

		match config.load() {
			Ok(()) => info!("Loaded config file successfully!"),
			Err(error) => warn!("Failed to load config file: {}", error),
		}

		return config;
	}

	pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
		let user_dirs = unwrap_or_return!(UserDirs::new(), Err("Failed to get user directory!".into()));
		let home_dir = user_dirs.home_dir();
		let config_dir = home_dir.join(Path::new(".argon/config.toml"));

		let config_toml = fs::read_to_string(config_dir)?;
		let config: GlobalConfig = toml::from_str(&config_toml)?;

		set_if_some!(self.host, config.host);
		set_if_some!(self.port, config.port);

		Ok(())
	}
}
