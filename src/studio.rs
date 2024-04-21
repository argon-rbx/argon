use anyhow::Result;
use roblox_install::RobloxStudio;
use std::{
	path::PathBuf,
	process::{Command, Stdio},
};

#[cfg(target_os = "windows")]
use winsafe::{co::SW, prelude::user_Hwnd, EnumWindows};

pub fn launch(path: Option<PathBuf>) -> Result<()> {
	let studio_path = RobloxStudio::locate()?.application_path().to_owned();

	Command::new(studio_path)
		.arg(path.unwrap_or_default())
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()?;

	Ok(())
}

#[allow(unused_variables)]
pub fn is_running(title: Option<String>) -> Result<bool> {
	#[cfg(target_os = "macos")]
	{
		let output = Command::new("osascript")
				.args([
					"-e",
					"tell app \"System Events\" to get the title of every window of (processes whose background only is false)",
				])
				.output()?;

		let windows = String::from_utf8(output.stdout)?;

		if let Some(title) = title {
			Ok(windows.contains(&format!("{} - Roblox Studio", title)))
		} else {
			Ok(windows.contains("Roblox Studio"))
		}
	}

	#[cfg(target_os = "windows")]
	{
		let is_studio_running = EnumWindows(|hwnd| -> bool {
			if !hwnd.IsWindowVisible() {
				return true;
			}

			if let Ok(text) = hwnd.GetWindowText() {
				if let Some(title) = &title {
					if text == format!("{} - Roblox Studio", title) {
						return false;
					}
				} else if text.contains("Roblox Studio") {
					return false;
				}
			}

			true
		})
		.is_err();

		Ok(is_studio_running)
	}

	#[cfg(target_os = "linux")]
	{
		anyhow::bail!("This feature is not yet supported on Linux!");
	}
}

#[allow(unused_variables)]
pub fn bring_to_front(title: Option<String>) -> Result<()> {
	#[cfg(target_os = "macos")]
	{
		// TODO: support title

		Command::new("osascript")
			.args([
				"-e",
				"tell application \"System Events\" to tell process \"RobloxStudio\" to set frontmost to true",
			])
			.output()?;

		Ok(())
	}

	#[cfg(target_os = "windows")]
	{
		EnumWindows(|hwnd| -> bool {
			if !hwnd.IsWindowVisible() {
				return true;
			}

			if let Ok(text) = hwnd.GetWindowText() {
				if let Some(title) = &title {
					if text == format!("{} - Roblox Studio", title) {
						hwnd.SetForegroundWindow();
						hwnd.ShowWindow(SW::RESTORE);

						return false;
					}
				} else if text.contains("Roblox Studio") {
					hwnd.SetForegroundWindow();
					hwnd.ShowWindow(SW::RESTORE);

					return false;
				}
			}

			true
		})?;

		Ok(())
	}

	#[cfg(target_os = "linux")]
	{
		anyhow::bail!("This feature is not yet supported on Linux!");
	}
}
