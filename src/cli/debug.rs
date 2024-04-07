use anyhow::Result;
use clap::Parser;

#[cfg(not(target_os = "linux"))]
use keybd_event::{KeyBondingInstance, KeyboardKey};

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "windows")]
use winsafe::{co::SW, prelude::user_Hwnd, EnumWindows};

use crate::exit;

/// Start or stop Roblox playtest with selected mode
#[derive(Parser)]
pub struct Debug {
	/// Debug mode to use (Play, Run, Start, Stop)
	#[arg()]
	mode: Option<String>,
}

impl Debug {
	pub fn main(self) -> Result<()> {
		let mode = self.mode.unwrap_or(String::from("Play"));

		if let Some(mode) = DebugMode::from_str(&mode) {
			if !bring_studio_to_front(None)? {
				exit!("There is no running Roblox Studio instance!");
			}

			send_keys(&mode);
		} else {
			exit!("Invalid debug mode!");
		}

		Ok(())
	}
}

#[allow(unused_variables)]
fn bring_studio_to_front(name: Option<&str>) -> Result<bool> {
	#[cfg(target_os = "macos")]
	{
		let output = Command::new("osascript")
			.args([
				"-e",
				"tell app \"System Events\" to return name of processes whose background only is false",
			])
			.output()?;

		let windows = String::from_utf8(output.stdout)?;
		let is_studio_running = windows.contains("RobloxStudio");

		if is_studio_running {
			Command::new("osascript")
				.args([
					"-e",
					"tell application \"System Events\" to tell process \"RobloxStudio\" to set frontmost to true",
				])
				.output()?;
		}

		Ok(is_studio_running)
	}

	#[cfg(target_os = "windows")]
	{
		Ok(EnumWindows(|hwnd| -> bool {
			if !hwnd.IsWindowVisible() {
				return true;
			}

			if let Ok(title) = hwnd.GetWindowText() {
				if name.is_some_and(|name| title == format!("{} - Roblox Studio", name))
					|| title.contains("Roblox Studio")
				{
					hwnd.SetForegroundWindow();
					hwnd.ShowWindow(SW::RESTORE);

					return false;
				}
			}

			true
		})
		.is_err())
	}

	#[cfg(target_os = "linux")]
	{
		anyhow::bail!("This feature is not yet supported on Linux!");
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
