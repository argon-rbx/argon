use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::{fs, path::PathBuf};

use crate::{argon_info, config::Config, ext::PathExt, installer, util};

/// Install Argon Roblox Studio plugin locally
#[derive(Parser)]
pub struct Plugin {
	/// Whether to `install` or `uninstall` the plugin
	#[arg(hide_possible_values = true)]
	mode: Option<PluginMode>,
	/// Custom plugin installation path
	#[arg()]
	path: Option<PathBuf>,
}

impl Plugin {
	pub fn main(self) -> Result<()> {
		let plugin_path = if let Some(path) = self.path {
			let smart_paths = Config::new().smart_paths;

			if path.is_dir() || (smart_paths && (path.extension().is_none())) {
				if !smart_paths || path.get_name().to_lowercase() != "argon" {
					path.join("Argon.rbxm")
				} else {
					path.with_extension("rbxm")
				}
			} else {
				path
			}
		} else {
			util::get_plugin_path()?
		};

		match self.mode.unwrap_or_default() {
			PluginMode::Install => {
				argon_info!("Installing Argon plugin..");
				installer::install_plugin(&plugin_path, true)?;
			}
			PluginMode::Uninstall => {
				argon_info!("Uninstalling Argon plugin..");
				fs::remove_file(plugin_path)?;
			}
		}

		Ok(())
	}
}

#[derive(Clone, Default, ValueEnum)]
enum PluginMode {
	#[default]
	Install,
	Uninstall,
}
