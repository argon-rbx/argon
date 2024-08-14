use anyhow::Result;
use clap::{Parser, ValueEnum};

use crate::{argon_error, argon_info, config::Config, updater};

/// Forcefully update Argon CLI and plugin if available
#[derive(Parser)]
pub struct Update {
	/// Whether to update `cli` or `plugin` or `both`
	#[arg(hide_possible_values = true)]
	mode: Option<UpdateMode>,
}

impl Update {
	pub fn main(self) -> Result<()> {
		let config = Config::new();

		let (cli, plugin) = match self.mode.unwrap_or_default() {
			UpdateMode::Both => (true, config.install_plugin),
			UpdateMode::Cli => (true, false),
			UpdateMode::Plugin => (false, true),
		};

		match updater::force_update(cli, plugin) {
			Ok(updated) => {
				if !updated {
					argon_info!("Everything is up to date!");
				}
			}
			Err(err) => argon_error!("Failed to update Argon: {}", err),
		}

		Ok(())
	}
}

#[derive(Clone, Default, ValueEnum)]
enum UpdateMode {
	Cli,
	Plugin,
	#[default]
	Both,
}
