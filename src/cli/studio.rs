use anyhow::Result;
use clap::Parser;
use log::trace;
use roblox_install::RobloxStudio;
use std::process::{Command, Stdio};

/// Launch new instance of Roblox Studio
#[derive(Parser)]
pub struct Studio {}

impl Studio {
	pub fn main(self) -> Result<()> {
		trace!("Launching Roblox Studio!");

		let path = RobloxStudio::locate()?.application_path().to_owned();

		Command::new(path)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()?;

		Ok(())
	}
}
