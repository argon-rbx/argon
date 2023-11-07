use anyhow::Result;
use clap::Parser;
use open;
use std::{fs::File, path::Path};

use crate::{logger, utils};

/// Edit global config with default editor
#[derive(Parser)]
pub struct Config {}

impl Config {
	pub fn main(self) -> Result<()> {
		let home_dir = utils::get_home_dir()?;

		let config_dir = home_dir.join(Path::new(".argon/config.toml"));

		if !config_dir.exists() {
			let create_config = logger::prompt("Config does not exist. Would you like to create one?", true);

			if create_config {
				File::create(&config_dir)?;
			} else {
				return Ok(());
			}
		}

		open::that(config_dir)?;

		Ok(())
	}
}
