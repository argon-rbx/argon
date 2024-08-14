use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};

#[cfg(not(target_os = "linux"))]
use keybd_event::{KeyBondingInstance, KeyboardKey};

use crate::studio;

/// Start or stop Roblox playtest with selected mode
#[derive(Parser)]
pub struct Debug {
	/// Debug mode to use (`play`, `run`, `start` or `stop`)
	#[arg(hide_possible_values = true)]
	mode: Option<DebugMode>,
}

impl Debug {
	pub fn main(self) -> Result<()> {
		if !studio::is_running(None)? {
			bail!("There is no running Roblox Studio instance!");
		}

		studio::focus(None)?;
		send_keys(self.mode.unwrap_or_default());

		Ok(())
	}
}

#[allow(unused_variables)]
fn send_keys(mode: DebugMode) {
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

#[derive(Clone, Default, ValueEnum)]
enum DebugMode {
	#[default]
	Play,
	Run,
	Start,
	Stop,
}
