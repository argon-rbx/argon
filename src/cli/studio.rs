use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::{argon_info, config::Config, ext::PathExt, studio};

/// Launch a new Roblox Studio instance
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
	pub fn main(mut self) -> Result<()> {
		if self.check && studio::is_running(None)? {
			argon_info!("Roblox Studio is already running!");
			return Ok(());
		}

		argon_info!("Launching Roblox Studio..");

		if let Some(path) = self.path.as_ref() {
			if Config::new().smart_paths && !path.exists() {
				let rbxl = path.with_file_name(path.get_name().to_owned() + ".rbxl");
				let rbxlx = path.with_file_name(path.get_name().to_owned() + ".rbxlx");

				if rbxl.exists() {
					self.path = Some(rbxl);
				} else if rbxlx.exists() {
					self.path = Some(rbxlx)
				}
			}
		}

		studio::launch(self.path)?;

		Ok(())
	}
}
