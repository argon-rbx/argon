use anyhow::{bail, Result};
use clap::Parser;

#[cfg(not(target_os = "linux"))]
use keybd_event::{KeyBondingInstance, KeyboardKey};

use crate::studio;

/// Start or stop Roblox playtest with selected mode
#[derive(Parser)]
pub struct Debug {
	/// Debug mode to use (Play, Run, Start, Stop)
	#[arg()]
	mode: Option<String>,
}

impl Debug {
	pub fn main(self) -> Result<()> {
		let mode = self.mode.unwrap_or(String::from("play"));

		if let Some(mode) = DebugMode::from_str(&mode) {
			if !studio::is_running(None)? {
				bail!("There is no running Roblox Studio instance!");
			}

			studio::focus(None)?;
			send_keys(&mode);
		} else {
			bail!("Invalid debug mode!");
		}

		Ok(())
	}
}

#[allow(unused_variables)]
fn send_keys(mode: &DebugMode) {
	#[cfg(not(target_os = "linux"))]
	{
		let mut kb = KeyBondingInstance::new().unwrap();

		match mode {
			DebugMode::Play => {
				kb.add_key(KeyboardKey::KeyF5);
			}
			DebugMode::Run => {
				kb.add_key(KeyboardKey::KeyF8);
			}
			DebugMode::Start => {
				kb.add_key(KeyboardKey::KeyF7);
			}
			DebugMode::Stop => {
				kb.has_shift(true);
				kb.add_key(KeyboardKey::KeyF5);
			}
		}

		kb.launching();
	}

	#[cfg(target_os = "linux")]
	{
		panic!("This feature is not yet supported on Linux!");
	}
}

enum DebugMode {
	Play,
	Run,
	Start,
	Stop,
}

impl DebugMode {
	fn from_str(mode: &str) -> Option<Self> {
		match mode.to_lowercase().as_str() {
			"play" => Some(Self::Play),
			"run" => Some(Self::Run),
			"start" => Some(Self::Start),
			"stop" => Some(Self::Stop),
			_ => None,
		}
	}
}
