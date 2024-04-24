use anyhow::Result;
use colored::Colorize;
use include_dir::{include_dir, Dir};
use log::trace;
use self_update::{backends::github::Update, self_replace, update::UpdateStatus};
use std::{env, fs, path::Path};

use crate::{
	argon_error, argon_info,
	ext::PathExt,
	logger, updater,
	util::{self, get_plugin_path},
};

const PLACE_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/place");
const PLUGIN_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/plugin");
const PACKAGE_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/package");
const MODEL_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/model");
const QUICK_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/quick");

const ARGON_PLUGIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Argon.rbxm"));

pub fn is_aftman() -> bool {
	let path = match env::current_exe() {
		Ok(path) => path,
		Err(_) => return false,
	};

	path.contains(&[".aftman", "tool-storage"])
}

pub fn verify(is_aftman: bool, with_plugin: bool) -> Result<()> {
	let argon_dir = util::get_argon_dir()?;
	let templates_dir = argon_dir.join("templates");

	if !argon_dir.exists() {
		fs::create_dir(&argon_dir)?;
	}

	if !templates_dir.exists() {
		fs::create_dir(&templates_dir)?;
	}

	if !is_aftman {
		let bin_dir = argon_dir.join("bin");

		if !bin_dir.exists() {
			fs::create_dir(&bin_dir)?;
		}

		globenv::set_path(&bin_dir.to_string())?;

		#[cfg(not(target_os = "windows"))]
		let exe_path = bin_dir.join("argon");

		#[cfg(target_os = "windows")]
		let exe_path = bin_dir.join("argon.exe");

		if !exe_path.exists() {
			fs::copy(env::current_exe()?, &exe_path)?;

			let remove_exe = logger::prompt("Installation completed! Do you want to remove this executable?", true);

			if remove_exe {
				self_replace::self_delete()?;
			}
		}
	}

	let place_template = templates_dir.join("place");
	let plugin_template = templates_dir.join("plugin");
	let package_template = templates_dir.join("package");
	let model_template = templates_dir.join("model");
	let quick_template = templates_dir.join("quick");

	if !place_template.exists() {
		fs::create_dir(&place_template)?;
		install_template(&PLACE_TEMPLATE, &place_template)?;
	}

	if !plugin_template.exists() {
		fs::create_dir(&plugin_template)?;
		install_template(&PLUGIN_TEMPLATE, &plugin_template)?;
	}

	if !package_template.exists() {
		fs::create_dir(&package_template)?;
		install_template(&PACKAGE_TEMPLATE, &package_template)?;
	}

	if !model_template.exists() {
		fs::create_dir(&model_template)?;
		install_template(&MODEL_TEMPLATE, &model_template)?;
	}

	if !quick_template.exists() {
		fs::create_dir(&quick_template)?;
		install_template(&QUICK_TEMPLATE, &quick_template)?;
	}

	if with_plugin {
		let plugin_path = get_plugin_path()?;

		if !plugin_path.exists() {
			install_plugin(&plugin_path, false)?;
		}
	}

	Ok(())
}

pub fn install_plugin(path: &Path, show_progress: bool) -> Result<()> {
	fs::create_dir_all(path.get_parent())?;

	let style = util::get_progress_style();

	let update = Update::configure()
		.repo_owner("argon-rbx")
		.repo_name("argon-roblox")
		.bin_name("Argon.rbxm")
		.target("")
		.show_download_progress(show_progress)
		.set_progress_style(style.0, style.1)
		.bin_install_path(path)
		.build()?;

	match update.download() {
		Ok(status) => match status {
			UpdateStatus::Updated(release) => {
				argon_info!("Installed Argon plugin, version: {}", release.version.bold());

				if path.contains(&["Roblox", "Plugins"]) {
					let mut status = updater::get_status()?;
					status.plugin_version = release.version;

					updater::set_staus(&status)?;
				}
			}
			_ => unreachable!(),
		},
		Err(err) => {
			trace!("Failed to install Argon plugin from GitHub: {}", err);

			if ARGON_PLUGIN.is_empty() {
				argon_error!("No internet connection! Failed to install Argon plugin - no bundled binary found");
				return Ok(());
			}

			fs::write(path, ARGON_PLUGIN)?;

			argon_info!("No internet connection! Installed Argon plugin from bundled binary")
		}
	}

	Ok(())
}

fn install_template(template: &Dir, path: &Path) -> Result<()> {
	for file in template.files() {
		fs::write(path.join(file.path()), file.contents())?;
	}

	for dir in template.dirs() {
		fs::create_dir(&path.join(dir.path()))?;
		install_template(dir, path)?;
	}

	Ok(())
}
