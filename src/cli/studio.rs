use anyhow::Result;
use clap::Parser;
use roblox_install::RobloxStudio;
use std::{
	path::PathBuf,
	process::{Command, Stdio},
};

use crate::argon_info;

/// Launch new instance of Roblox Studio
#[derive(Parser)]
pub struct Studio {
	/// Path to place or model to open
	#[arg()]
	path: Option<PathBuf>,
}

impl Studio {
	pub fn main(self) -> Result<()> {
		let studio_path = RobloxStudio::locate()?.application_path().to_owned();

		argon_info!("Launching Roblox Studio");

		Command::new(studio_path)
			.arg(self.path.unwrap_or_default())
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()?;

		Ok(())
	}
}
