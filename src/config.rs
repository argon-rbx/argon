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
	/// Include documentation in the project (README, CHANGELOG, etc.)
	pub include_docs: bool,
	/// Use git for source control
	pub use_git: bool,
	/// Use Wally for package management
	pub use_wally: bool,

	/// Run Argon asynchronously, freeing up the terminal
	pub run_async: bool,
	/// Scan for the first available port if selected one is in use
	pub scan_ports: bool,
	/// Automatically detect project type
	pub detect_project: bool,
	/// Always run commands with sourcemap generation
	pub with_sourcemap: bool,
	/// Build using XML format by default
	pub build_xml: bool,

	/// Check for new Argon releases on startup
	pub check_updates: bool,
	/// Automatically install Argon updates if available
	pub auto_update: bool,
	/// Install Roblox plugin locally and keep it updated
	pub install_plugin: bool,

	/// Use Rojo namespace by default
	pub rojo_mode: bool,
	/// Use roblox-ts by default
	pub ts_mode: bool,

	/// Package manager to use when running roblox-ts scripts (npm, yarn, etc.)
	pub package_manager: String,
	/// Use .lua file extension instead of .luau when writing scripts
	pub lua_extension: bool,
	/// Move files to the bin instead of deleting them (two-way sync)
	pub move_to_bin: bool,
	/// Share anonymous Argon usage statistics with the community
	pub share_stats: bool,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			host: String::from("localhost"),
			port: 8000,
			template: String::from("place"),
			license: String::from("Apache-2.0"),
			include_docs: true,
			use_git: true,
			use_wally: false,

			run_async: false,
			scan_ports: true,
			detect_project: true,
			with_sourcemap: false,
			build_xml: false,

			check_updates: true,
			auto_update: false,
			install_plugin: true,

			rojo_mode: false,
			ts_mode: false,

			package_manager: String::from("npm"),
			lua_extension: false,
			move_to_bin: false,
			share_stats: true,
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

		let load_result = config.merge_toml();

		CONFIG.set(config).expect("Config already loaded");

		load_result
	}

	pub fn save(&self) -> Result<()> {
		let path = util::get_argon_dir()?.join("config.toml");

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
			if let Ok(doc) = Self::get_field_docs(setting) {
				table.add_row(vec![setting.to_owned(), default.to_string(), doc.trim().to_owned()]);
			}
		}

		table
	}

	fn merge_toml(&mut self) -> Result<()> {
		let path = util::get_argon_dir()?.join("config.toml");
		let config = toml::from_str(&fs::read_to_string(path)?)?;

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
