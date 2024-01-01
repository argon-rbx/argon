use anyhow::Result;
use log::{info, warn};
use optfield::optfield;
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

use crate::util;

#[optfield(GlobalConfig, attrs, merge_fn)]
#[derive(Serialize, Deserialize)]
pub struct Config {
	pub host: String,
	pub port: u16,
	pub source_dir: String,
	pub template: String,
	pub auto_init: bool,
	pub git_init: bool,
	pub rojo_mode: bool,

	#[serde(skip)]
	pub project_name: String,
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
			template: String::from("default"),
			auto_init: false,
			git_init: true,
			rojo_mode: false,

			project_name: String::from(".argon"),
			src: String::from(".src"),
			data: String::from(".data"),
		};

		match config.load_global() {
			Ok(()) => info!("Config file loaded"),
			Err(err) => warn!("Failed to load config file: {}", err),
		}

		config
	}

	pub fn load_global(&mut self) -> Result<()> {
		let home_dir = util::get_home_dir()?;
		let config_dir = home_dir.join(".argon").join("config.toml");

		let config_toml = fs::read_to_string(config_dir)?;
		let config: GlobalConfig = toml::from_str(&config_toml)?;

		self.merge_opt(config);

		if self.rojo_mode {
			self.project_name = String::from("default");
			self.src = String::from("init");
			self.data = String::from("meta");
		}

		Ok(())
	}
}
