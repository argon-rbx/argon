use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::{argon_info, studio};

/// Launch a new instance of Roblox Studio
#[derive(Parser)]
pub struct Studio {
	/// Path to place or model to open
	#[arg()]
	path: Option<PathBuf>,

	/// Check if Roblox Studio is already running
	#[arg(short, long)]
	check: bool,
}

impl Studio {
	pub fn main(self) -> Result<()> {
		if self.check && studio::is_running(None)? {
			argon_info!("Roblox Studio is already running!");
			return Ok(());
		}

		argon_info!("Launching Roblox Studio..");

		studio::launch(self.path)?;

		Ok(())
	}
}
