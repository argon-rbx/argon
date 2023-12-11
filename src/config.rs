use anyhow::Result;
use log::{info, warn};
use optfield::optfield;
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

use crate::utils;

#[optfield(GlobalConfig, attrs, merge_fn)]
#[derive(Serialize, Deserialize)]
pub struct Config {
	pub host: String,
	pub port: u16,
	pub source_dir: String,
	pub project_name: String,
	pub template: String,
	pub auto_init: bool,
	pub git_init: bool,
	pub rojo_mode: bool,

	#[serde(skip)]
	pub src: String,
	#[serde(skip)]
	pub data: String,
}

impl Config {
	pub fn load() -> Self {
		let mut config = Self {
			host: String::from("localhost"),
			port: 8000,
			source_dir: String::from("src"),
			project_name: String::from(".argon"),
			template: String::from("default"),
			auto_init: false,
			git_init: true,
			rojo_mode: false,
			src: String::from(".src"),
			data: String::from(".data"),
		};

		match config.load_global() {
			Ok(()) => info!("Config file loaded"),
			Err(error) => warn!("Failed to load config file: {}", error),
		}

		config
	}

	pub fn load_global(&mut self) -> Result<()> {
		let home_dir = utils::get_home_dir()?;
		let config_dir = home_dir.join(".argon").join("config.toml");

		let config_toml = fs::read_to_string(config_dir)?;
		let config: GlobalConfig = toml::from_str(&config_toml)?;

		self.merge_opt(config);

		if self.rojo_mode {
			self.src = String::from("init");
			self.data = String::from("meta");
		}

		Ok(())
	}
}
