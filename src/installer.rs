use anyhow::Result;
use std::{env, fs};

use crate::{logger, util};

const GAME_PROJECT: &str = include_str!("../assets/templates/game/project.json");
const GAME_GITIGNORE: &str = include_str!("../assets/templates/game/.gitignore");
const GAME_README: &str = include_str!("../assets/templates/game/README.md");

const PLUGIN_PROJECT: &str = include_str!("../assets/templates/plugin/project.json");
const PLUGIN_GITIGNORE: &str = include_str!("../assets/templates/plugin/.gitignore");
const PLUGIN_README: &str = include_str!("../assets/templates/plugin/README.md");
const PLUGIN_LICENSE: &str = include_str!("../assets/templates/plugin/LICENSE.md");
const PLUGIN_CHANGELOG: &str = include_str!("../assets/templates/plugin/CHANGELOG.md");

const PACKAGE_PROJECT: &str = include_str!("../assets/templates/package/project.json");
const PACKAGE_GITIGNORE: &str = include_str!("../assets/templates/package/.gitignore");
const PACKAGE_README: &str = include_str!("../assets/templates/package/README.md");
const PACKAGE_LICENSE: &str = include_str!("../assets/templates/package/LICENSE.md");
const PACKAGE_CHANGELOG: &str = include_str!("../assets/templates/package/CHANGELOG.md");
const PACKAGE_WALLY: &str = include_str!("../assets/templates/package/wally.toml");

const MODEL_PROJECT: &str = include_str!("../assets/templates/model/project.json");
const MODEL_GITIGNORE: &str = include_str!("../assets/templates/model/.gitignore");
const MODEL_README: &str = include_str!("../assets/templates/model/README.md");
const MODEL_LICENSE: &str = include_str!("../assets/templates/model/LICENSE.md");

pub fn install() -> Result<()> {
	let home_dir = util::get_home_dir()?;

	let argon_dir = home_dir.join(".argon");
	let bin_dir = argon_dir.join("bin");
	let templates_dir = argon_dir.join("templates");

	if !argon_dir.exists() {
		fs::create_dir(&argon_dir)?;
	}

	if !bin_dir.exists() {
		fs::create_dir(&bin_dir)?;
	}

	if !templates_dir.exists() {
		fs::create_dir(&templates_dir)?;
	}

	globenv::set_path(bin_dir.to_str().unwrap())?;

	#[cfg(not(target_os = "windows"))]
	let exe_path = bin_dir.join("argon");

	#[cfg(target_os = "windows")]
	let exe_path = bin_dir.join("argon.exe");

	if !exe_path.exists() {
		let current_exe = env::current_exe()?;

		fs::copy(current_exe, &exe_path)?;

		let remove_exe = logger::prompt("Installation completed! Do you want to remove this executable?", true);

		if remove_exe {
			self_replace::self_delete()?;
		}
	}

	let game_template = templates_dir.join("game");
	let plugin_template = templates_dir.join("plugin");
	let package_template = templates_dir.join("package");
	let model_template = templates_dir.join("model");

	if !game_template.exists() {
		fs::create_dir(&game_template)?;

		let project = game_template.join("project.json");
		let gitignore = game_template.join(".gitignore");
		let readme = game_template.join("README.md");

		fs::write(project, GAME_PROJECT)?;
		fs::write(gitignore, GAME_GITIGNORE)?;
		fs::write(readme, GAME_README)?;
	}

	if !plugin_template.exists() {
		fs::create_dir(&plugin_template)?;

		let project = plugin_template.join("project.json");
		let gitignore = plugin_template.join(".gitignore");
		let readme = plugin_template.join("README.md");
		let license = plugin_template.join("LICENSE.md");
		let changelog = plugin_template.join("CHANGELOG.md");

		fs::write(project, PLUGIN_PROJECT)?;
		fs::write(gitignore, PLUGIN_GITIGNORE)?;
		fs::write(readme, PLUGIN_README)?;
		fs::write(license, PLUGIN_LICENSE)?;
		fs::write(changelog, PLUGIN_CHANGELOG)?;
	}

	if !package_template.exists() {
		fs::create_dir(&package_template)?;

		let project = package_template.join("project.json");
		let gitignore = package_template.join(".gitignore");
		let readme = package_template.join("README.md");
		let license = package_template.join("LICENSE.md");
		let changelog = package_template.join("CHANGELOG.md");
		let wally = package_template.join("wally.toml");

		fs::write(project, PACKAGE_PROJECT)?;
		fs::write(gitignore, PACKAGE_GITIGNORE)?;
		fs::write(readme, PACKAGE_README)?;
		fs::write(license, PACKAGE_LICENSE)?;
		fs::write(changelog, PACKAGE_CHANGELOG)?;
		fs::write(wally, PACKAGE_WALLY)?;
	}

	if !model_template.exists() {
		fs::create_dir(&model_template)?;

		let project = model_template.join("project.json");
		let gitignore = model_template.join(".gitignore");
		let readme = model_template.join("README.md");
		let license = model_template.join("LICENSE.md");

		fs::write(project, MODEL_PROJECT)?;
		fs::write(gitignore, MODEL_GITIGNORE)?;
		fs::write(readme, MODEL_README)?;
		fs::write(license, MODEL_LICENSE)?;
	}

	Ok(())
}
