use anyhow::Result;
use clap::Parser;

use crate::{argon_error, argon_info, config::Config, updater};

/// Forcefully update Argon CLI and plugin if available
#[derive(Parser)]
pub struct Update {
	/// Update the Argon CLI only
	#[clap(short, long)]
	pub cli: bool,
	/// Update the Argon plugin only
	#[clap(short, long)]
	pub plugin: bool,
}

impl Update {
	pub fn main(mut self) -> Result<()> {
		let config = Config::new();

		if !self.cli && !self.plugin {
			self.cli = true;
			self.plugin = config.install_plugin;
		}

		match updater::force_update(self.cli, self.plugin) {
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
