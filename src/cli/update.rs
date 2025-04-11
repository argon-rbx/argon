use anyhow::Result;
use clap::{Parser, ValueEnum};

use crate::{argon_error, argon_info, config::Config, updater};

/// Forcefully update Argon components if available
#[derive(Debug, Parser)]
pub struct Update {
	/// Update mode
	#[clap(short, long, value_enum, default_value_t = UpdateMode::All)]
	pub mode: UpdateMode,

	/// Force update
	#[clap(short, long)]
	pub force: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum UpdateMode {
	/// Update everything
	All,
	/// Update only the CLI
	Cli,
	/// Update only the plugin
	Plugin,
	/// Update only the templates
	Templates,
	/// Update only the VS Code extension
	Vscode,
}

impl Update {
	pub fn run(&self) -> Result<()> {
		match self.mode {
			UpdateMode::All => {
				updater::manual_update(true, true, true, true, self.force)?;
			}
			UpdateMode::Cli => {
				updater::manual_update(true, false, false, false, self.force)?;
			}
			UpdateMode::Plugin => {
				updater::manual_update(false, true, false, false, self.force)?;
			}
			UpdateMode::Templates => {
				updater::manual_update(false, false, true, false, self.force)?;
			}
			UpdateMode::Vscode => {
				updater::manual_update(false, false, false, true, self.force)?;
			}
		}

		Ok(())
	}
}
