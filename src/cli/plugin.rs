use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::{fs, path::PathBuf};

use crate::{argon_info, config::Config, installer, util};

/// Install Argon Roblox Studio plugin locally
#[derive(Parser)]
pub struct Plugin {
	/// Whether to `install` or `uninstall` the plugin
	#[arg(hide_possible_values = true)]
	action: Action,
	/// Custom plugin installation path
	#[arg()]
	path: Option<PathBuf>,
}

impl Plugin {
	pub fn main(self) -> Result<()> {
		let plugin_path = if let Some(path) = self.path {
			if path.is_dir() || (Config::new().smart_paths && path.extension().is_none()) {
				path.join("Argon.rbxm")
			} else {
				path
			}
		} else {
			util::get_plugin_path()?
		};

		match self.action {
			Action::Install => {
				argon_info!("Installing Argon plugin..");
				installer::install_plugin(&plugin_path, true)?;
			}
			Action::Uninstall => {
				argon_info!("Uninstalling Argon plugin..");
				fs::remove_file(plugin_path)?;
			}
		}

		Ok(())
	}
}

#[derive(ValueEnum, Clone)]
enum Action {
	/// Install the plugin in the selected path or in Studio's plugin directory
	Install,
	/// Uninstall the plugin from the selected path or from Studio's plugin directory
	Uninstall,
}
