use anyhow::{anyhow, bail, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use open;
use std::{env, fs::File, path::PathBuf};

use crate::{
	argon_info,
	config::{Config as ArgonConfig, ConfigKind},
	ext::PathExt,
	logger, util,
};

/// Edit global or workspace config with editor or CLI
#[derive(Parser)]
pub struct Config {
	/// Setting to change (if left empty config will be opened)
	#[arg()]
	setting: Option<String>,

	/// Value to set setting to (if left empty default value will be used)
	#[arg()]
	value: Option<String>,

	/// List all available settings
	#[arg(short, long)]
	list: bool,

	/// Restore all settings to default values
	#[arg(short, long)]
	default: bool,

	/// Export current config to the custom file
	#[arg(short, long)]
	export: Option<PathBuf>,

	/// Which config file to work with (`global` or `workspace`)
	#[arg(short, long, hide_possible_values = true)]
	config: Option<ConfigType>,
}

impl Config {
	pub fn main(self) -> Result<()> {
		let config = ArgonConfig::new();

		let config_kind = match self.config.unwrap_or_default() {
			ConfigType::Default => ConfigKind::Default,
			ConfigType::Global => ConfigKind::Global(util::get_argon_dir()?.join("config.toml")),
			ConfigType::Workspace => ConfigKind::Workspace(env::current_dir()?.join("argon.toml")),
		};

		if *config.kind() == ConfigKind::Default
			|| (config_kind != ConfigKind::Default && *config.kind() != config_kind)
		{
			drop(config);
			ArgonConfig::load_virtual(config_kind)?;
		} else {
			drop(config);
		};

		let config = ArgonConfig::new();

		if self.list {
			argon_info!(
				"List of all available config options:\n\n{}\nVisit {} to learn more details!",
				config.list(),
				"https://argon.wiki/docs/configuration#global-config".bold()
			);

			return Ok(());
		}

		let config_path = config
			.kind()
			.path()
			.ok_or(anyhow!("Resolve all config errors before using this command!"))?
			.to_owned();

		if self.default {
			if config_path.exists() {
				File::create(config_path)?;
			}

			argon_info!(
				"Restored all settings to default values in {} config",
				config.kind().to_string().bold()
			);

			return Ok(());
		}

		if let Some(path) = self.export {
			config.save(&path)?;

			argon_info!(
				"Exported {} to {} config",
				config.kind().to_string().bold(),
				path.to_string().bold()
			);

			return Ok(());
		}

		match (self.setting, self.value) {
			(Some(setting), Some(value)) => {
				drop(config);
				let mut config = ArgonConfig::new_mut();

				if config.has_setting(&setting) {
					if let Err(err) = config.set(&setting, &value) {
						bail!("Failed to parse value: {}", err);
					}

					config.save(&config_path)?;

					argon_info!(
						"Set {} setting to {} in {} config",
						setting.bold(),
						value.bold(),
						config.kind().to_string().bold()
					);
				} else {
					bail!("Setting {} does not exist", setting.bold());
				}
			}
			(Some(setting), None) => {
				let default = ArgonConfig::default();

				if default.has_setting(&setting) {
					drop(config);
					let mut config = ArgonConfig::new_mut();

					config
						.set(&setting, &default.get(&setting).unwrap().to_string())
						.unwrap();

					config.save(&config_path)?;

					argon_info!(
						"Set {} to its default value in {} config",
						setting.bold(),
						config.kind().to_string().bold()
					);
				} else {
					bail!("Setting {} does not exist", setting.bold());
				}
			}
			_ => {
				if !config_path.exists() {
					let create_config = logger::prompt(
						&format!(
							"{} config does not exist. Would you like to create one?",
							config.kind().to_string().bold()
						),
						true,
					);

					if create_config {
						File::create(&config_path)?;
					} else {
						return Ok(());
					}
				}

				argon_info!("Opened config file. Manually go to: {}", config_path.to_string().bold());

				open::that(config_path)?;
			}
		}

		Ok(())
	}
}

#[derive(Clone, Default, ValueEnum, PartialEq)]
enum ConfigType {
	#[default]
	Default,
	Global,
	Workspace,
}
