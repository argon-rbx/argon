use anyhow::{Context, Result};
use colored::Colorize;
use directories::UserDirs;
use env_logger::WriteStyle;
use json_formatter::JsonFormatter;
use log::LevelFilter;
use rbx_dom_weak::types::Variant;
use rbx_reflection::ClassTag;
use roblox_install::RobloxStudio;
use std::{env, path::PathBuf, process::Command};

use crate::Properties;

/// Returns the `.argon` directory
pub fn get_argon_dir() -> Result<PathBuf> {
	let user_dirs = UserDirs::new().context("Failed to get user directory")?;

	Ok(user_dirs.home_dir().join(".argon"))
}

/// Returns the Git or local username of the current user
pub fn get_username() -> String {
	if let Ok(output) = Command::new("git").arg("config").arg("user.name").output() {
		let username = String::from_utf8_lossy(&output.stdout).trim().to_owned();

		if !username.is_empty() {
			return username;
		}
	}

	whoami::username()
}

pub fn get_plugin_path() -> Result<PathBuf> {
	Ok(RobloxStudio::locate()?.plugins_path().join("Argon.rbxm"))
}

/// Checks if the given `class` is a service
pub fn is_service(class: &str) -> bool {
	let descriptor = rbx_reflection_database::get().classes.get(class);

	let has_tag = if let Some(descriptor) = descriptor {
		descriptor.tags.contains(&ClassTag::Service)
	} else {
		false
	};

	has_tag || class == "StarterPlayerScripts" || class == "StarterCharacterScripts"
}

/// Checks if the given `class` is a script
pub fn is_script(class: &str) -> bool {
	class == "Script" || class == "LocalScript" || class == "ModuleScript"
}

/// Kills the process with the given `pid`
pub fn kill_process(pid: u32) {
	#[cfg(not(target_os = "windows"))]
	{
		// Kill main process
		Command::new("kill").arg(pid.to_string()).output().ok();

		// Kill child processes
		Command::new("pkill").arg("-P").arg(pid.to_string()).output().ok();
	}

	// Kill both main and child processes
	#[cfg(target_os = "windows")]
	Command::new("TASKKILL")
		.arg("/F")
		.arg("/T")
		.args(["/PID", &pid.to_string()])
		.output()
		.ok();
}

pub fn process_exists(pid: u32) -> bool {
	#[cfg(not(target_os = "windows"))]
	{
		if let Ok(output) = Command::new("kill").arg("-0").arg(pid.to_string()).output() {
			output.status.success()
		} else {
			false
		}
	}

	#[cfg(target_os = "windows")]
	{
		let output = Command::new("TASKLIST")
			.arg("/NH")
			.args(["/FI", &format!("PID eq {}", pid)])
			.output();

		if let Ok(output) = output {
			String::from_utf8_lossy(&output.stdout).contains("argon.exe")
		} else {
			false
		}
	}
}

/// Returns progress bar styling
pub fn get_progress_style() -> (String, String) {
	let mut template = match env_log_style() {
		WriteStyle::Always => "PROGRESS: ".magenta().bold().to_string(),
		_ => "PROGRESS: ".into(),
	};
	template.push_str("[{bar:40}] ({bytes}/{total_bytes})");

	(template, String::from("=>-"))
}

/// Returns the `RUST_VERBOSE` environment variable
pub fn env_verbosity() -> LevelFilter {
	let verbosity = env::var("RUST_VERBOSE").unwrap_or("ERROR".into());

	match verbosity.as_str() {
		"OFF" => LevelFilter::Off,
		"ERROR" => LevelFilter::Error,
		"WARN" => LevelFilter::Warn,
		"INFO" => LevelFilter::Info,
		"DEBUG" => LevelFilter::Debug,
		"TRACE" => LevelFilter::Trace,
		_ => LevelFilter::Error,
	}
}

/// Returns the `RUST_LOG_STYLE` environment variable
pub fn env_log_style() -> WriteStyle {
	let log_style = env::var("RUST_LOG_STYLE").unwrap_or("auto".into());

	match log_style.as_str() {
		"always" => WriteStyle::Always,
		"never" => WriteStyle::Never,
		_ => WriteStyle::Auto,
	}
}

/// Returns the `RUST_BACKTRACE` environment variable
pub fn env_backtrace() -> bool {
	let backtrace = env::var("RUST_BACKTRACE").unwrap_or("0".into());
	backtrace == "1"
}

/// Returns the `RUST_YES` environment variable
pub fn env_yes() -> bool {
	let yes = env::var("RUST_YES").unwrap_or("0".into());
	yes == "1"
}

/// Returns line of code count from snapshot's properties
pub fn count_loc_from_properties(properties: &Properties) -> usize {
	let mut loc = 0;

	for value in properties.values() {
		if let Variant::String(value) = value {
			loc += value.lines().count();
		}
	}

	loc
}

/// Returns a custom serde_json formatter
pub fn get_json_formatter() -> JsonFormatter<'static> {
	JsonFormatter::new()
		.with_array_breaks(false)
		.with_extra_newline(true)
		.with_max_decimals(4)
}
