use anyhow::Result;
use clap::Parser;

use crate::{argon_error, argon_info, config::Config, updater};

/// Forcefully update Argon CLI and plugin if available
#[derive(Parser)]
pub struct Update {}

impl Update {
	pub fn main(self) -> Result<()> {
		let config = Config::new();

		match updater::force_update(config.install_plugin) {
			Ok(()) => argon_info!("Successfully updated Argon! Everything is up to date"),
			Err(err) => argon_error!("Failed to update Argon: {}", err),
		}

		Ok(())
	}
}
