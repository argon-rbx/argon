use anyhow::Result;
use clap::Parser;
use roblox_install::RobloxStudio;
use std::process::{Command, Stdio};

use crate::argon_info;

/// Launch new instance of Roblox Studio
#[derive(Parser)]
pub struct Studio {}

impl Studio {
	pub fn main(self) -> Result<()> {
		let path = RobloxStudio::locate()?.application_path().to_owned();

		argon_info!("Launching Roblox Studio");

		Command::new(path)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()?;

		Ok(())
	}
}
