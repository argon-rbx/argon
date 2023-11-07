use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{
	fs,
	path::{Path, PathBuf},
};
use toml;

use crate::utils;

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
	project: Option<PathBuf>,
	auto_init: Option<bool>,
	git_init: Option<bool>,
}

pub struct Config {
	pub host: String,
	pub port: u16,
	pub project: PathBuf,
	pub auto_init: bool,
	pub git_init: bool,
}

impl Config {
	pub fn new() -> Config {
		let mut config = Config {
			host: String::from("localhost"),
			port: 8000,
			project: PathBuf::from(".argon"),
			auto_init: false,
			git_init: true,
		};

		match config.load() {
			Ok(()) => info!("Config file loaded"),
			Err(error) => warn!("Failed to load config file: {}", error),
		}

		return config;
	}

	pub fn load(&mut self) -> Result<()> {
		let home_dir = utils::get_home_dir()?;
		let config_dir = home_dir.join(Path::new(".argon/config.toml"));

		let config_toml = fs::read_to_string(config_dir)?;
		let config: GlobalConfig = toml::from_str(&config_toml)?;

		set_if_some!(self.host, config.host);
		set_if_some!(self.port, config.port);
		set_if_some!(self.project, config.project);
		set_if_some!(self.auto_init, config.auto_init);
		set_if_some!(self.git_init, config.git_init);

		Ok(())
	}
}
