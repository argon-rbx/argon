use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use open;
use std::{
	env,
	fs::{self, File},
	path::PathBuf,
};

use crate::{argon_info, config::Config as ArgonConfig, ext::PathExt, logger, util};

/// Edit global config with default editor or CLI
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

	/// Which config file to work with (`global` or `workspace`)
	#[arg(short, long, hide_possible_values = true)]
	config: Option<ConfigType>,

	/// Save current config to the custom file
	#[arg(short, long)]
	output: Option<PathBuf>,
}

impl Config {
	pub fn main(self) -> Result<()> {
		if self.list {
			argon_info!(
				"List of all available config options:\n\n{}\nVisit {} to learn more details!",
				ArgonConfig::list(),
				"https://argon.wiki/docs/configuration#global-config".bold()
			);

			return Ok(());
		}

		match self.config.unwrap_or_default() {
			ConfigType::Global => ArgonConfig::load_global(),
			ConfigType::Workspace => ArgonConfig::load_workspace(&env::current_dir()?),
			_ => {}
		}

		let config = ArgonConfig::new();
		let config_path = if let Some(path) = config.kind().path() {
			path.to_owned()
		} else {
			util::get_argon_dir()?.join("config.toml")
		};

		if self.default {
			if config_path.exists() {
				if config.move_to_bin {
					trash::delete(config_path)?;
				} else {
					fs::remove_file(config_path)?;
				}
			}

			argon_info!("Restored all settings to default values");

			return Ok(());
		}

		let output = self.output.unwrap_or(config_path);

		match (self.setting, self.value) {
			(Some(setting), Some(value)) => {
				drop(config);
				let mut config = ArgonConfig::new_mut();

				if config.has_setting(&setting) {
					if let Err(err) = config.set(&setting, &value) {
						bail!("Failed to parse value: {}", err);
					}

					config.save(&output)?;

					argon_info!("Set {} to {}", setting.bold(), value.bold());
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

					config.save(&output)?;

					argon_info!("Set {} to its default value", setting.bold());
				} else {
					bail!("Setting {} does not exist", setting.bold());
				}
			}
			_ => {
				// TODO: fix this when there is workspace config
				if !output.exists() {
					let create_config = logger::prompt(
						&format!(
							"{} config does not exist. Would you like to create one?",
							config.kind().to_string().bold()
						),
						true,
					);

					if create_config {
						File::create(&output)?;
					} else {
						return Ok(());
					}
				}

				argon_info!("Opened config file. Manually go to: {}", output.to_string().bold());

				open::that(output)?;
			}
		}

		Ok(())
	}
}

#[derive(Clone, Default, ValueEnum)]
enum ConfigType {
	#[default]
	Default,
	Global,
	Workspace,
}
