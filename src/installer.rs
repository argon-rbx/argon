use anyhow::Result;
use colored::Colorize;
use include_dir::{include_dir, Dir};
use log::trace;
use rbx_dom_weak::{types::Variant, Ustr};
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
const EMPTY_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/empty");

const ARGON_PLUGIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Argon.rbxm"));

pub fn is_managed() -> bool {
	let path = match env::current_exe() {
		Ok(path) => path,
		Err(_) => return false,
	};

	!path.contains(&[".argon", "bin"]) && (path.contains(&["bin"]) || path.contains(&["tool-storage"]))
}

pub fn verify(is_managed: bool, with_plugin: bool) -> Result<()> {
	if !is_managed {
		let bin_dir = util::get_argon_dir()?.join("bin");

		if !bin_dir.exists() {
			fs::create_dir_all(&bin_dir)?;
		}

		globenv::set_path(&bin_dir.to_string())?;

		#[cfg(not(target_os = "windows"))]
		let exe_path = bin_dir.join("argon");

		#[cfg(target_os = "windows")]
		let exe_path = bin_dir.join("argon.exe");

		if !exe_path.exists() {
			fs::copy(env::current_exe()?, &exe_path)?;

			if logger::prompt("Installation completed! Do you want to remove this executable?", true) {
				self_replace::self_delete()?;
			}
		}
	}

	install_templates(false)?;

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
		.repo_owner("LupaHQ")
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

					updater::set_status(&status)?;
				}
			}
			_ => unreachable!(),
		},
		Err(err) => {
			trace!("Failed to install Argon plugin from GitHub: {}", err);

			#[allow(clippy::const_is_empty)]
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

pub fn install_templates(update: bool) -> Result<()> {
	let templates_dir = util::get_argon_dir()?.join("templates");

	let place_template = templates_dir.join("place");
	let plugin_template = templates_dir.join("plugin");
	let package_template = templates_dir.join("package");
	let model_template = templates_dir.join("model");
	let quick_template = templates_dir.join("quick");
	let empty_template = templates_dir.join("empty");

	if update || !place_template.exists() {
		fs::create_dir_all(&place_template)?;
		install_template(&PLACE_TEMPLATE, &place_template)?;
	}

	if update || !plugin_template.exists() {
		fs::create_dir_all(&plugin_template)?;
		install_template(&PLUGIN_TEMPLATE, &plugin_template)?;
	}

	if update || !package_template.exists() {
		fs::create_dir_all(&package_template)?;
		install_template(&PACKAGE_TEMPLATE, &package_template)?;
	}

	if update || !model_template.exists() {
		fs::create_dir_all(&model_template)?;
		install_template(&MODEL_TEMPLATE, &model_template)?;
	}

	if update || !quick_template.exists() {
		fs::create_dir_all(&quick_template)?;
		install_template(&QUICK_TEMPLATE, &quick_template)?;
	}

	if update || !empty_template.exists() {
		fs::create_dir_all(&empty_template)?;
		install_template(&EMPTY_TEMPLATE, &empty_template)?;
	}

	Ok(())
}

fn install_template(template: &Dir, path: &Path) -> Result<()> {
	for file in template.files() {
		if file.path().get_name() != ".gitkeep" {
			fs::write(path.join(file.path()), file.contents())?;
		}
	}

	for dir in template.dirs() {
		fs::create_dir_all(path.join(dir.path()))?;
		install_template(dir, path)?;
	}

	Ok(())
}

pub fn get_plugin_version() -> String {
	// May seem hacky, but this function will only be
	// called once for most users and is non-critical anyway
	if let Ok(dom) = rbx_binary::from_reader(ARGON_PLUGIN) {
		for (_, instance) in dom.into_raw().1 {
			if instance.name == "manifest" && instance.class == "ModuleScript" {
				if let Some(Variant::String(source)) = instance.properties.get(&Ustr::from("Source")) {
					let source = &source[source.find(r#"["version"] = ""#).unwrap_or(0) + 15..];
					return source[..source.find(r#"","#).unwrap_or(6)].to_owned();
				}
			}
		}
	}

	String::from("0.0.0")
}
