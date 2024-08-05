use anyhow::Result;
use colored::Colorize;
use log::{debug, trace, warn};
use self_update::{backends::github::Update, cargo_crate_version, version::bump_is_greater};
use serde::{Deserialize, Serialize};
use std::{fs, time::SystemTime};

use crate::{
	argon_error, argon_info, logger,
	util::{self, get_plugin_path},
};

#[derive(Serialize, Deserialize)]
pub struct UpdateStatus {
	pub last_checked: SystemTime,
	pub plugin_version: String,
}

pub fn get_status() -> Result<UpdateStatus> {
	let path = util::get_argon_dir()?.join("update.toml");

	if path.exists() {
		match toml::from_str(&fs::read_to_string(&path)?) {
			Ok(status) => return Ok(status),
			Err(_) => warn!("Update status file is corrupted! Creating new one.."),
		}
	}

	let status = UpdateStatus {
		last_checked: SystemTime::UNIX_EPOCH,
		plugin_version: String::from("0.0.0"),
	};

	fs::write(path, toml::to_string(&status)?)?;

	Ok(status)
}

pub fn set_status(status: &UpdateStatus) -> Result<()> {
	let path = util::get_argon_dir()?.join("update.toml");

	fs::write(path, toml::to_string(status)?)?;

	Ok(())
}

fn update_cli(prompt: bool) -> Result<bool> {
	let style = util::get_progress_style();
	let current_version = cargo_crate_version!();

	let update = Update::configure()
		.repo_owner("argon-rbx")
		.repo_name("argon")
		.bin_name("argon")
		.show_download_progress(true)
		.set_progress_style(style.0, style.1)
		.build()?;

	let release = update.get_latest_release()?;

	if bump_is_greater(current_version, &release.version)? {
		if !prompt
			|| logger::prompt(
				&format!(
					"New Argon version: {} is available! Would you like to update?",
					release.version.bold()
				),
				true,
			) {
			if !prompt {
				argon_info!("New Argon version: {} is available! Updating..", release.version.bold());
			}

			match update.update() {
				Ok(_) => {
					argon_info!(
						"CLI updated! Restart the program to apply changes. Visit {} to read the changelog",
						"https://argon.wiki/changelog/argon".bold()
					);
					return Ok(true);
				}
				Err(err) => argon_error!("Failed to update Argon: {}", err),
			}
		} else {
			trace!("Argon is out of date!");
		}
	} else {
		trace!("Argon is up to date!");
	}

	Ok(false)
}

fn update_plugin(status: &mut UpdateStatus, prompt: bool) -> Result<bool> {
	let style = util::get_progress_style();
	let current_version = &status.plugin_version;
	let plugin_path = get_plugin_path()?;

	let update = Update::configure()
		.repo_owner("argon-rbx")
		.repo_name("argon-roblox")
		.bin_name("Argon.rbxm")
		.target("")
		.show_download_progress(true)
		.set_progress_style(style.0, style.1)
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
				true,
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
						"Roblox plugin updated! Make sure you have {} setting enabled to see changes. Visit {} to read the changelog",
						"Reload plugins on file changed".bold(),
						"https://argon.wiki/changelog/argon-roblox".bold()
					);

					status.plugin_version = release.version;
					return Ok(true);
				}
				Err(err) => argon_error!("Failed to update Argon plugin: {}", err),
			}
		} else {
			trace!("Argon plugin is out of date!");
		}
	} else {
		trace!("Argon plugin is up to date!");
	}

	Ok(false)
}

pub fn check_for_updates(with_plugin: bool, should_prompt: bool) -> Result<()> {
	let mut status = get_status()?;

	if status.last_checked.elapsed()?.as_secs() < 3600 {
		debug!("Update check already performed within the last hour");
		return Ok(());
	}

	update_cli(should_prompt)?;

	if with_plugin {
		update_plugin(&mut status, should_prompt)?;
	}

	status.last_checked = SystemTime::now();
	set_status(&status)?;

	Ok(())
}

pub fn force_update(cli: bool, plugin: bool) -> Result<bool> {
	let mut status = get_status()?;
	let mut updated = false;

	if cli && update_cli(false)? {
		updated = true;
	}

	if plugin && update_plugin(&mut status, false)? {
		updated = true;
	}

	status.last_checked = SystemTime::now();
	set_status(&status)?;

	Ok(updated)
}
