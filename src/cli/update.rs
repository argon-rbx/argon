use anyhow::Result;
use clap::{Parser, ValueEnum};

use crate::{argon_error, argon_info, config::Config, updater};

/// Forcefully update Argon components if available
#[derive(Parser)]
pub struct Update {
	/// Whether to update `cli`, `plugin`, `templates` or `all`
	#[arg(hide_possible_values = true)]
	mode: Option<UpdateMode>,
	/// Whether to force update even if there is no newer version
	#[arg(short, long)]
	force: bool,
}

impl Update {
	pub fn main(self) -> Result<()> {
		let config = Config::new();

		let (cli, plugin, templates) = match self.mode.unwrap_or_default() {
			UpdateMode::All => (true, config.install_plugin, config.update_templates),
			UpdateMode::Cli => (true, false, false),
			UpdateMode::Plugin => (false, true, false),
			UpdateMode::Templates => (false, false, true),
		};

		match updater::manual_update(cli, plugin, templates, self.force) {
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
	Templates,
	#[default]
	All,
}
