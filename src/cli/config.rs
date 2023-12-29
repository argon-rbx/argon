use anyhow::Result;
use clap::Parser;
use open;
use std::fs::File;

use crate::{logger, util};

/// Edit global config with default editor
#[derive(Parser)]
pub struct Config {}

impl Config {
	pub fn main(self) -> Result<()> {
		let home_dir = util::get_home_dir()?;

		let config_path = home_dir.join(".argon").join("config.toml");

		if !config_path.exists() {
			let create_config = logger::prompt("Config does not exist. Would you like to create one?", true);

			if create_config {
				File::create(&config_path)?;
			} else {
				return Ok(());
			}
		}

		open::that(config_path)?;

		Ok(())
	}
}
