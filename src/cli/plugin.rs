use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::{argon_info, installer, util};

/// Install Argon Roblox Studio plugin locally
#[derive(Parser)]
pub struct Plugin {
	/// Custom plugin installation path
	#[arg()]
	path: Option<PathBuf>,
}

impl Plugin {
	pub fn main(self) -> Result<()> {
		let plugin_path = if let Some(path) = self.path {
			path
		} else {
			util::get_plugin_path()?
		};

		argon_info!("Installing Argon plugin..");

		installer::install_plugin(&plugin_path, true)?;

		Ok(())
	}
}
