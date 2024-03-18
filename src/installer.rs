use anyhow::Result;
use include_dir::{include_dir, Dir};
use std::{env, fs, path::Path};

use crate::{ext::PathExt, logger, util};

const PLACE_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/place");
const PLUGIN_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/plugin");
const PACKAGE_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/package");
const MODEL_TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/model");

fn is_aftman() -> Result<bool> {
	if let Some(parent) = env::current_exe()?.parent() {
		if !parent.ends_with("bin") {
			return Ok(false);
		}

		if let Some(parent) = parent.parent() {
			if parent.ends_with(".aftman") {
				return Ok(true);
			}
		}
	}

	Ok(false)
}

pub fn install() -> Result<()> {
	let home_dir = util::get_home_dir()?;

	let argon_dir = home_dir.join(".argon");
	let templates_dir = argon_dir.join("templates");

	if !argon_dir.exists() {
		fs::create_dir(&argon_dir)?;
	}

	if !templates_dir.exists() {
		fs::create_dir(&templates_dir)?;
	}

	if !is_aftman()? {
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
