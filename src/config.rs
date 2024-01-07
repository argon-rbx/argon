use anyhow::{bail, Result};
use colored::Colorize;
use documented::DocumentedFields;
use log::{info, warn};
use optfield::optfield;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::{
	fs,
	ops::{Index, IndexMut},
};
use toml;

use crate::util;

macro_rules! make_fn {
	($name:ident, String) => {
		pub fn $name(&self) -> &String {
			match &self.$name {
				ConfigField::String(value) => &value,
				_ => panic!("An internal error occurred in Config macro"),
			}
		}
	};
	($name:ident, bool) => {
		pub fn $name(&self) -> bool {
			match &self.$name {
				ConfigField::Bool(value) => *value,
				_ => panic!("An internal error occurred in Config macro"),
			}
		}
	};
	($name:ident, u16) => {
		pub fn $name(&self) -> u16 {
			match &self.$name {
				ConfigField::Int(value) => *value,
				_ => panic!("An internal error occurred in Config macro"),
			}
		}
	};
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum ConfigField {
	String(String),
	Bool(bool),
	Int(u16),
	None,
}

// To add new Config value:
//
// 1. Add field to Config struct
// 2. Set default value in get_defaults()
// 3. Add getter function using make_fn! macro
// 4. Add field to the Index implementation
// 5. Add field to the IndexMut implementation
// 6. Add field to the IntoIterator implementation
//
// last 3 steps will be replaced with the derive in the future

#[optfield(GlobalConfig, attrs, merge_fn)]
#[derive(Deserialize, DocumentedFields)]
pub struct Config {
	/// Default server host name; localhost
	host: ConfigField,
	/// Default server port; 8000
	port: ConfigField,
	/// Default source directory; src
	source_dir: ConfigField,
	/// Default project template; game
	template: ConfigField,
	/// Whether to spawn the Argon child process; true
	spawn: ConfigField,
	/// Whether to automatically initialize the project; false
	auto_init: ConfigField,
	/// Whether to use git for project management; true
	use_git: ConfigField,
	/// Whether to include documentation in the project; true
	include_docs: ConfigField,
	/// Whether to use Rojo mode; false
	rojo_mode: ConfigField,

	#[serde(skip)]
	project_name: String,
	#[serde(skip)]
	src: String,
	#[serde(skip)]
	data: String,
}

impl Serialize for Config {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(None)?;
		let defaults = Self::load_default();

		for (k, v) in self {
			if v == &defaults[&k] {
				continue;
			}

			map.serialize_entry(&k, v)?;
		}

		map.end()
	}
}

impl Config {
	pub fn load_default() -> Self {
		Self {
			host: ConfigField::String(String::from("localhost")),
			port: ConfigField::Int(8000),
			source_dir: ConfigField::String(String::from("src")),
			template: ConfigField::String(String::from("game")),
			spawn: ConfigField::Bool(true),
			auto_init: ConfigField::Bool(false),
			use_git: ConfigField::Bool(true),
			include_docs: ConfigField::Bool(true),
			rojo_mode: ConfigField::Bool(false),

			project_name: String::from(".argon"),
			src: String::from(".src"),
			data: String::from(".data"),
		}
	}

	pub fn load_global(&mut self) -> Result<()> {
		let home_dir = util::get_home_dir()?;
		let config_dir = home_dir.join(".argon").join("config.toml");

		let config_toml = fs::read_to_string(config_dir)?;
		let config: GlobalConfig = toml::from_str(&config_toml)?;

		self.merge_opt(config);

		if self.rojo_mode() {
			self.project_name = String::from("default");
			self.src = String::from("init");
			self.data = String::from("meta");
		}

		Ok(())
	}

	pub fn load() -> Self {
		let mut config = Self::load_default();

		match config.load_global() {
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
		self[setting] != ConfigField::None
	}

	pub fn set(&mut self, setting: &str, value: &str) -> Result<()> {
		match self[setting] {
			ConfigField::String(_) => {
				self[setting] = ConfigField::String(value.into());
			}
			ConfigField::Bool(_) => {
				self[setting] = ConfigField::Bool(value.parse()?);
			}
			ConfigField::Int(_) => {
				self[setting] = ConfigField::Int(value.parse()?);
			}
			ConfigField::None => {
				bail!("Setting '{}' does not exist", setting)
			}
		}

		Ok(())
	}

	pub fn list() -> String {
		let defaults = Self::load_default();
		let mut settings = String::new();

		settings.push_str(&format!(
			"| {0: <15} | {1: <10} | {2: <50} |\n",
			"Setting".bold(),
			"Default".bold(),
			"Description".bold()
		));
		settings.push_str(&format!(
			"| {0: <15} | {1: <10} | {2: <50} |\n",
			"-".repeat(15),
			"-".repeat(10),
			"-".repeat(50)
		));

		for (field, _) in &defaults {
			let doc: Vec<_> = Self::get_field_comment(&field).unwrap().split(';').collect();

			settings.push_str(&format!(
				"| {0: <15} | {1: <10} | {2: <50} |\n",
				&field,
				doc[1].trim(),
				doc[0].trim()
			));
		}

		settings.pop();

		settings
	}

	pub fn project_name(&self) -> &String {
		&self.project_name
	}

	pub fn src(&self) -> &String {
		&self.src
	}

	pub fn data(&self) -> &String {
		&self.data
	}

	make_fn!(host, String);
	make_fn!(port, u16);
	make_fn!(source_dir, String);
	make_fn!(template, String);
	make_fn!(spawn, bool);
	make_fn!(auto_init, bool);
	make_fn!(use_git, bool);
	make_fn!(include_docs, bool);
	make_fn!(rojo_mode, bool);
}

impl Index<&str> for Config {
	type Output = ConfigField;

	fn index(&self, index: &str) -> &Self::Output {
		match index {
			"host" => &self.host,
			"port" => &self.port,
			"source_dir" => &self.source_dir,
			"template" => &self.template,
			"spawn" => &self.spawn,
			"auto_init" => &self.auto_init,
			"use_git" => &self.use_git,
			"include_docs" => &self.include_docs,
			"rojo_mode" => &self.rojo_mode,
			_ => &ConfigField::None,
		}
	}
}

impl IndexMut<&str> for Config {
	fn index_mut(&mut self, index: &str) -> &mut ConfigField {
		match index {
			"host" => &mut self.host,
			"port" => &mut self.port,
			"source_dir" => &mut self.source_dir,
			"template" => &mut self.template,
			"spawn" => &mut self.spawn,
			"auto_init" => &mut self.auto_init,
			"use_git" => &mut self.use_git,
			"include_docs" => &mut self.include_docs,
			"rojo_mode" => &mut self.rojo_mode,
			_ => panic!("Config field: {} does not exist!", index),
		}
	}
}

impl<'a> IntoIterator for &'a Config {
	type Item = (String, &'a ConfigField);
	type IntoIter = ConfigIntoIterator<'a>;

	fn into_iter(self) -> Self::IntoIter {
		ConfigIntoIterator { config: self, index: 0 }
	}
}

pub struct ConfigIntoIterator<'a> {
	config: &'a Config,
	index: usize,
}

impl<'a> Iterator for ConfigIntoIterator<'a> {
	type Item = (String, &'a ConfigField);

	fn next(&mut self) -> Option<Self::Item> {
		let result = match self.index {
			0 => "host",
			1 => "port",
			2 => "source_dir",
			3 => "template",
			4 => "spawn",
			5 => "auto_init",
			6 => "use_git",
			7 => "include_docs",
			8 => "rojo_mode",
			_ => return None,
		};

		self.index += 1;

		Some((String::from(result), &self.config[result]))
	}
}
