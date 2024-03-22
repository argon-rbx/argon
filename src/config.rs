use anyhow::Result;
use config_derive::{Get, Iter, Set, Val};
use documented::DocumentedFields;
use optfield::optfield;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::{fs, sync::OnceLock};
use toml;

use crate::{logger::Table, util};

static CONFIG: OnceLock<Config> = OnceLock::new();

#[optfield(GlobalConfig, merge_fn, attrs = (derive(Deserialize)))]
#[derive(Debug, Clone, Deserialize, DocumentedFields, Val, Iter, Get, Set)]
pub struct Config {
	/// Default server host name
	pub host: String,
	/// Default server port number
	pub port: u16,
	/// Default project template (place, model, etc.)
	pub template: String,
	/// Default project license (SPDX identifier)
	pub license: String,
	/// Run Argon asynchronously, freeing up the terminal
	pub run_async: bool,
	/// Scan for the first available port if selected one is in use
	pub scan_ports: bool,
	/// Check for new Argon releases on startup
	pub check_updates: bool,
	/// Automatically install Argon updates if available
	pub auto_update: bool,
	/// Install Roblox plugin locally and keep it updated
	pub install_plugin: bool,
	/// Share anonymous Argon usage statistics with the community
	pub share_stats: bool,
	/// Automatically detect project type
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
			run_async: false,
			scan_ports: true,
			check_updates: true,
			auto_update: false,
			install_plugin: true,
			share_stats: true,
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
	pub fn new() -> &'static Self {
		CONFIG.get().expect("Config not loaded!")
	}

	/// This should only be used by the `config` CLI command
	pub fn new_mut() -> Self {
		Self::new().clone()
	}

	/// This sould be called once, at the start of the program
	pub fn load() -> Result<()> {
		let mut config = Self::default();

		let load_result = config.read_toml();

		CONFIG.set(config).expect("Config already loaded");

		load_result
	}

	pub fn save(&self) -> Result<()> {
		let home_dir = util::get_home_dir()?;
		let path = home_dir.join(".argon").join("config.toml");

		fs::write(path, toml::to_string(self)?)?;

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
		let path = home_dir.join(".argon").join("config.toml");

		let config_toml = fs::read_to_string(path)?;
		let config = toml::from_str(&config_toml)?;

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
