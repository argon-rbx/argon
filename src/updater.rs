use anyhow::Result;
use colored::Colorize;
use log::{debug, trace, warn};
use self_update::{backends::github::Update, cargo_crate_version, version::bump_is_greater};
use serde::{Deserialize, Serialize};
use std::{
	env::consts::{ARCH, OS},
	fs,
	sync::Once,
	time::SystemTime,
};

use crate::{
	argon_error, argon_info,
	constants::TEMPLATES_VERSION,
	installer::{get_plugin_version, install_templates},
	logger,
	util::{self, get_plugin_path},
};

static UPDATE_FORCED: Once = Once::new();

#[derive(Serialize, Deserialize)]
pub struct UpdateStatus {
	pub last_checked: SystemTime,
	pub plugin_version: String,
	pub templates_version: u8,
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
		plugin_version: get_plugin_version(),
		templates_version: TEMPLATES_VERSION,
	};

	fs::write(path, toml::to_string(&status)?)?;

	Ok(status)
}

pub fn set_status(status: &UpdateStatus) -> Result<()> {
	let path = util::get_argon_dir()?.join("update.toml");

	fs::write(path, toml::to_string(status)?)?;

	Ok(())
}

fn update_cli(prompt: bool, force: bool) -> Result<bool> {
	let style = util::get_progress_style();
	let current_version = cargo_crate_version!();

	let target = {
		// Windows automatically translates x86_64 programs to aarch64
		if OS == "windows" && ARCH == "aarch64" {
			"windows-x86_64"
		} else {
			&format!("{}-{}", OS, ARCH)
		}
	};

	let update = Update::configure()
		.repo_owner("argon-rbx")
		.repo_name("argon")
		.bin_name("argon")
		.target(target)
		.show_download_progress(true)
		.set_progress_style(style.0, style.1)
		.build()?;

	let release = update.get_latest_release()?;

	if bump_is_greater(current_version, &release.version)? || force {
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

fn update_plugin(status: &mut UpdateStatus, prompt: bool, force: bool) -> Result<bool> {
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

	if bump_is_greater(current_version, &release.version)? || force {
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

fn update_templates(status: &mut UpdateStatus, prompt: bool, force: bool) -> Result<bool> {
	if status.templates_version < TEMPLATES_VERSION || force {
		if !prompt || logger::prompt("Default templates have changed! Would you like to update?", true) {
			if !prompt {
				argon_info!("Default templates have changed! Updating..",);
			}

			install_templates(true)?;

			status.templates_version = TEMPLATES_VERSION;

			return Ok(true);
		} else {
			trace!("Templates are out of date!");
		}
	} else {
		trace!("Project templates are up to date!");
	}

	Ok(false)
}

pub fn check_for_updates(plugin: bool, templates: bool, prompt: bool) -> Result<()> {
	let mut status = get_status()?;

	if UPDATE_FORCED.is_completed() {
		return Ok(());
	}

	if status.last_checked.elapsed()?.as_secs() < 3600 {
		debug!("Update check already performed within the last hour");
		return Ok(());
	}

	update_cli(prompt, false)?;

	if plugin {
		update_plugin(&mut status, prompt, false)?;
	}

	if templates {
		update_templates(&mut status, prompt, false)?;
	}

	status.last_checked = SystemTime::now();
	set_status(&status)?;

	Ok(())
}

pub fn manual_update(cli: bool, plugin: bool, templates: bool, force: bool) -> Result<bool> {
	UPDATE_FORCED.call_once(|| {});

	let mut status = get_status()?;
	let mut updated = false;

	if cli && update_cli(false, force)? {
		updated = true;
	}

	if plugin && update_plugin(&mut status, false, force)? {
		updated = true;
	}

	if templates && update_templates(&mut status, false, force)? {
		updated = true;
	}

	status.last_checked = SystemTime::now();
	set_status(&status)?;

	Ok(updated)
}
