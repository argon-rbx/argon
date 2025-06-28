use anyhow::Result;
use colored::Colorize;
use config_derive::{Get, Iter, Set, Val};
use documented::DocumentedFields;
use lazy_static::lazy_static;
use log::{debug, info};
use optfield::optfield;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::{
	env,
	fmt::{self, Debug, Display, Formatter},
	fs, mem,
	path::{Path, PathBuf},
	sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use toml;

use crate::{argon_error, logger::Table, util};

lazy_static! {
	static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub enum ConfigKind {
	#[default]
	Default,
	Global(PathBuf),
	Workspace(PathBuf),
}

#[optfield(OptConfig, merge_fn, attrs = (derive(Deserialize)))]
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
	/// Use Git for source control
	pub use_git: bool,
	/// Use Wally for package management
	pub use_wally: bool,
	/// Use selene for codebase linting
	pub use_selene: bool,

	/// Run Argon asynchronously, freeing up the terminal
	pub run_async: bool,
	/// Scan for the first available port if selected one is in use
	pub scan_ports: bool,
	/// Automatically detect project type
	pub detect_project: bool,
	/// Use smart path resolver when running commands
	pub smart_paths: bool,
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
	/// Update default project templates when available
	pub update_templates: bool,

	/// Use Rojo namespace by default
	pub rojo_mode: bool,
	/// Use roblox-ts by default
	pub ts_mode: bool,

	/// Automatically rename corrupted instances when syncing back
	pub rename_instances: bool,
	/// Keep duplicate instances (by adding UUID suffixes) when syncing back
	pub keep_duplicates: bool,
	/// Move files to the bin instead of deleting them (two-way sync)
	pub move_to_bin: bool,
	/// Number of changes allowed before prompting user for confirmation
	pub changes_threshold: usize,
	/// Maximum number of unsynced changes before showing a warning
	pub max_unsynced_changes: usize,

	/// Use .lua file extension instead of .luau when writing scripts
	pub lua_extension: bool,
	/// Ignore line endings when reading files to avoid script diffs
	pub ignore_line_endings: bool,
	/// Package manager to use when running roblox-ts scripts (npm, bun, etc.)
	pub package_manager: String,
	/// Share anonymous Argon usage statistics with the community
	pub share_stats: bool,

	#[serde(skip)]
	/// Internal
	kind: ConfigKind,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			host: String::from("localhost"),
			port: 8000,
			template: String::from("place"),
			license: String::from("Apache-2.0"),
			include_docs: false,
			use_git: true,
			use_wally: false,
			use_selene: false,

			run_async: false,
			scan_ports: true,
			detect_project: true,
			smart_paths: false,
			with_sourcemap: false,
			build_xml: false,

			check_updates: true,
			auto_update: false,
			install_plugin: true,
			update_templates: true,

			rojo_mode: true,
			ts_mode: false,

			rename_instances: true,
			keep_duplicates: false,
			move_to_bin: false,
			changes_threshold: 5,
			max_unsynced_changes: 10,

			lua_extension: false,
			ignore_line_endings: true,
			package_manager: String::from("npm"),
			share_stats: true,

			kind: ConfigKind::default(),
		}
	}
}

impl ConfigKind {
	pub fn path(&self) -> Option<&Path> {
		match self {
			Self::Default => None,
			Self::Global(path) | Self::Workspace(path) => Some(path),
		}
	}
}

impl Config {
	pub fn new() -> RwLockReadGuard<'static, Self> {
		CONFIG.read().unwrap()
	}

	pub fn new_mut() -> RwLockWriteGuard<'static, Self> {
		CONFIG.try_write().expect("Failed to acquire write lock on config")
	}

	pub fn load() -> Result<ConfigKind> {
		let mut config = Self::default();

		let config_kind = || -> Result<ConfigKind> {
			let workspace_config = env::current_dir()?.join("argon.toml");
			let global_config = util::get_argon_dir()?.join("config.toml");

			let kind = if workspace_config.exists() {
				ConfigKind::Workspace(workspace_config)
			} else if global_config.exists() {
				ConfigKind::Global(global_config)
			} else {
				ConfigKind::Default
			};

			if let Some(path) = kind.path() {
				config.merge_opt(toml::from_str(&fs::read_to_string(path)?)?);
			}

			config.kind = kind.clone();

			Ok(kind)
		}();

		*CONFIG.write().unwrap() = config;

		config_kind
	}

	pub fn load_virtual(kind: ConfigKind) -> Result<()> {
		let kind = match kind {
			ConfigKind::Default => ConfigKind::Global(util::get_argon_dir()?.join("config.toml")),
			_ => kind,
		};

		if kind.path().unwrap().exists() {
			Self::load_specific(kind);
		} else {
			*CONFIG.write().unwrap() = Config {
				kind,
				..Default::default()
			};
		}

		Ok(())
	}

	pub fn load_workspace(path: &Path) {
		Self::load_specific(ConfigKind::Workspace(path.join("argon.toml")))
	}

	#[inline]
	fn load_specific(kind: ConfigKind) {
		if mem::discriminant(&kind) == mem::discriminant(&CONFIG.read().unwrap().kind) {
			debug!("{} config file already loaded", kind);
			return;
		}

		let path = kind.path().unwrap();

		if !path.exists() {
			debug!("{} config file not found", kind);
			return;
		}

		let mut config = Self::default();

		let load_result = || -> Result<()> {
			config.merge_opt(toml::from_str(&fs::read_to_string(path)?)?);

			config.kind = match kind {
				ConfigKind::Global(_) => ConfigKind::Global(path.to_owned()),
				ConfigKind::Workspace(_) => ConfigKind::Workspace(path.to_owned()),
				_ => ConfigKind::Default,
			};

			*CONFIG.write().unwrap() = config;

			Ok(())
		}();

		match load_result {
			Ok(()) => info!("{} config file loaded", kind),
			Err(err) => {
				argon_error!("Failed to load {} config file: {}", kind.to_string().bold(), err);
			}
		}
	}

	pub fn save(&self, path: &Path) -> Result<()> {
		fs::write(path, toml::to_string(self)?)?;

		Ok(())
	}

	pub fn has_setting(&self, setting: &str) -> bool {
		self.get(setting).is_some()
	}

	pub fn list(&self) -> Table {
		let defaults = Self::default();
		let mut table = Table::new();
		let defaults_only = self == &defaults;

		if defaults_only {
			table.set_header(vec!["Setting", "Default", "Description"]);
		} else {
			table.set_header(vec!["Setting", "Default", "Current", "Description"]);
		}

		for (setting, default) in &defaults {
			if let Ok(doc) = Self::get_field_docs(setting) {
				if defaults_only {
					table.add_row(vec![setting.to_owned(), default.to_string(), doc.trim().to_owned()]);
				} else {
					let default = default.to_string();
					let mut current = self.get(setting).map(|v| v.to_string()).unwrap();

					if current == default {
						current = String::new();
					}

					table.add_row(vec![setting.to_owned(), default, current, doc.trim().to_owned()]);
				}
			}
		}

		table
	}

	pub fn kind(&self) -> &ConfigKind {
		&self.kind
	}
}

impl Display for ConfigKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Default => "Default",
				Self::Global(_) => "Global",
				Self::Workspace(_) => "Workspace",
			}
		)
	}
}

impl PartialEq for Config {
	fn eq(&self, other: &Self) -> bool {
		for (k, v) in self {
			if other.get(k) != Some(v) {
				return false;
			}
		}

		true
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
