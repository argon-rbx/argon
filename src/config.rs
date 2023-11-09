use anyhow::Result;
use log::{info, warn};
use optfield::optfield;
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

use crate::utils;

macro_rules! set_if_some {
	($default:expr, $optional:expr) => {
		if $optional.is_some() {
			$default = $optional.unwrap();
		}
	};
}

#[optfield(GlobalConfig, attrs)]
#[derive(Serialize, Deserialize)]
pub struct Config {
	pub host: String,
	pub port: u16,
	pub project: String,
	pub template: String,
	pub auto_init: bool,
	pub git_init: bool,
}

impl Config {
	pub fn new() -> Config {
		let mut config = Config {
			host: String::from("localhost"),
			port: 8000,
			project: String::from(".argon"),
			template: String::from("default"),
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
		let config_dir = home_dir.join(".argon").join("config.toml");

		let config_toml = fs::read_to_string(config_dir)?;
		let config: GlobalConfig = toml::from_str(&config_toml)?;

		set_if_some!(self.host, config.host);
		set_if_some!(self.port, config.port);
		set_if_some!(self.project, config.project);
		set_if_some!(self.template, config.template);
		set_if_some!(self.auto_init, config.auto_init);
		set_if_some!(self.git_init, config.git_init);

		Ok(())
	}
}
