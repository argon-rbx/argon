use anyhow::Result;
use config_derive::{Get, Iter, Set, Val};
use documented::DocumentedFields;
use log::{info, warn};
use optfield::optfield;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::fs;
use toml;

use crate::{logger::Table, util};

// To add new Config value:
//
// 1. Add field to Config struct (with description)
// 2. Set default value in the Default impl

#[optfield(GlobalConfig, merge_fn, attrs = (derive(Deserialize)))]
#[derive(Clone, Deserialize, DocumentedFields, Val, Iter, Get, Set)]
pub struct Config {
	/// Default server host name
	pub host: String,
	/// Default server port
	pub port: u16,
	/// Default project template
	pub template: String,
	/// Default project license
	pub license: String,
	/// Spawn Argon as child process, freeing up the terminal
	pub spawn: bool,
	/// Scan for the first available port if the default is taken
	pub scan_ports: bool,
	/// Automatically detect if project is roblox-ts
	pub auto_detect: bool,
	/// Use git for source control
	pub use_git: bool,
	/// Use Wally for package management
	pub use_wally: bool,
	/// Include documentation in the project (README, LICENSE, etc.)
	pub include_docs: bool,
	/// Use Rojo namespace by default
	pub rojo_mode: bool,
	/// Use roblox-ts by default
	pub ts_mode: bool,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			host: String::from("localhost"),
			port: 8000,
			template: String::from("place"),
			license: String::from("Apache-2.0"),
			spawn: true,
			scan_ports: true,
			auto_detect: true,
			use_git: true,
			use_wally: false,
			include_docs: true,
			rojo_mode: false,
			ts_mode: false,
		}
	}
}

impl Config {
	pub fn load() -> Self {
		let mut config = Self::default();

		match config.read_toml() {
			Ok(()) => info!("Config file loaded"),
			Err(err) => warn!("Failed to load config file: {}", err),
		}

		config
	}

	pub fn save(&self) -> Result<()> {
		let home_dir = util::get_home_dir()?;
		let config_dir = home_dir.join(".argon").join("config.toml");

		fs::write(config_dir, toml::to_string(self)?)?;

		Ok(())
	}

	pub fn has_setting(&self, setting: &str) -> bool {
		self.get(setting).is_some()
	}

	pub fn list() -> Table {
		let defaults = Self::default();
		let mut table = Table::new();

		table.set_header(vec!["Setting", "Default", "Description"]);

		for (setting, default) in &defaults {
			if let Ok(doc) = Self::get_field_comment(setting) {
				table.add_row(vec![setting.to_owned(), default.to_string(), doc.trim().to_owned()]);
			}
		}

		table
	}

	fn read_toml(&mut self) -> Result<()> {
		let home_dir = util::get_home_dir()?;
		let config_dir = home_dir.join(".argon").join("config.toml");

		let config_toml = fs::read_to_string(config_dir)?;
		let config: GlobalConfig = toml::from_str(&config_toml)?;

		self.merge_opt(config);

		Ok(())
	}
}

impl Serialize for Config {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(None)?;
		let defaults = Self::default();

		for (k, v) in self {
			if v == defaults.get(k).unwrap() {
				continue;
			}

			map.serialize_entry(&k, &v)?;
		}

		map.end()
	}
}
