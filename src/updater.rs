use anyhow::Result;
use colored::Colorize;
use env_logger::WriteStyle;
use log::{debug, trace};
use roblox_install::RobloxStudio;
use self_update::{backends::github::Update, cargo_crate_version, version::bump_is_greater};
use serde::{Deserialize, Serialize};
use std::{fs, time::SystemTime};

use crate::{argon_error, argon_info, logger, util};

#[derive(Serialize, Deserialize)]
struct UpdateStatus {
	last_checked: SystemTime,
	plugin_version: String,
}

fn get_status() -> Result<UpdateStatus> {
	let home_dir = util::get_home_dir()?;
	let path = home_dir.join(".argon").join("update.toml");

	if path.exists() {
		let status_toml = fs::read_to_string(path)?;
		let status = toml::from_str(&status_toml)?;

		Ok(status)
	} else {
		let status = UpdateStatus {
			last_checked: SystemTime::UNIX_EPOCH,
			plugin_version: String::from("0.0.0"),
		};

		fs::write(path, toml::to_string(&status)?)?;

		Ok(status)
	}
}

fn update_staus(status: &UpdateStatus) -> Result<()> {
	let home_dir = util::get_home_dir()?;
	let path = home_dir.join(".argon").join("update.toml");

	fs::write(path, toml::to_string(status)?)?;

	Ok(())
}

fn get_progress_style() -> (String, String) {
	let mut template = match util::get_log_style() {
		WriteStyle::Always => "PROGRESS: ".magenta().bold().to_string(),
		_ => "PROGRESS: ".to_string(),
	};
	template.push_str("[{bar:40}] ({bytes}/{total_bytes})");

	(template, String::from("=>-"))
}

fn update_cli(prompt: bool) -> Result<()> {
	let style = get_progress_style();
	let current_version = cargo_crate_version!();

	let update = Update::configure()
		.repo_owner("argon-rbx")
		.repo_name("argon")
		.bin_name("argon")
		.no_confirm(true)
		.show_output(false)
		.show_download_progress(true)
		.set_progress_style(style.0, style.1)
		.current_version(current_version)
		.build()?;

	let release = update.get_latest_release()?;

	if bump_is_greater(current_version, &release.version)? {
		if !prompt
			|| logger::prompt(
				&format!(
					"New Argon version: {} is available! Would you like to update?",
					release.version.bold()
				),
				false,
			) {
			if !prompt {
				argon_info!("New Argon version: {} is available! Updating..", release.version.bold());
			}

			match update.update() {
				Ok(_) => argon_info!("Argon updated! Restart the program to apply changes"),
				Err(err) => argon_error!("Failed to update Argon: {}", err),
			}
		} else {
			trace!("Argon is out of date!");
		}
	} else {
		trace!("Argon is up to date!");
	}

	Ok(())
}

fn update_plugin(status: &mut UpdateStatus, prompt: bool) -> Result<()> {
	let style = get_progress_style();
	let current_version = &status.plugin_version;
	let plugin_path = RobloxStudio::locate()?.plugins_path().join("Argon.rbxm");

	let update = Update::configure()
		.repo_owner("argon-rbx")
		.repo_name("argon-roblox")
		.bin_name("Argon.rbxm")
		.target("")
		.no_confirm(true)
		.show_output(false)
		.show_download_progress(true)
		.set_progress_style(style.0, style.1)
		.current_version(current_version)
		.bin_install_path(plugin_path)
		.build()?;

	let release = update.get_latest_release()?;

	if bump_is_greater(current_version, &release.version)? {
		if !prompt
			|| logger::prompt(
				&format!(
					"New version of Argon plugin: {} is available! Would you like to update?",
					release.version.bold()
				),
				false,
			) {
			if !prompt {
				argon_info!(
					"New version of Argon plugin: {} is available! Updating..",
					release.version.bold()
				);
			}

			match update.download() {
				Ok(_) => {
					argon_info!(
						"Argon plugin updated! Make sure you have {} setting enabled to see changes",
						"Reload plugins on file changed".bold()
					);

					status.plugin_version = release.version;
				}
				Err(err) => argon_error!("Failed to update Argon plugin: {}", err),
			}
		} else {
			trace!("Argon plugin is out of date!");
		}
	} else {
		trace!("Argon plugin is up to date!");
	}

	Ok(())
}

pub fn check_for_updates(plugin: bool, prompt: bool) -> Result<()> {
	let mut status = get_status()?;

	if status.last_checked.elapsed()?.as_secs() < 3600 {
		debug!("Update check already performed within the last hour");
		return Ok(());
	}

	update_cli(prompt)?;

	if plugin {
		update_plugin(&mut status, prompt)?;
	}

	status.last_checked = SystemTime::now();

	update_staus(&status)?;

	Ok(())
}
