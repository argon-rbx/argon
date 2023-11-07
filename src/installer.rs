use anyhow::Result;
use std::{env, fs};

use crate::{logger, utils};

const DEFAULT_PROJECT: &str = include_str!("../assets/templates/default/.argon.project.json");
const DEFAULT_GITIGNORE: &str = include_str!("../assets/templates/default/.gitignore");
const DEFAULT_README: &str = include_str!("../assets/templates/default/README.md");

const COMPACT_PROJECT: &str = include_str!("../assets/templates/compact/.argon.project.json");

pub fn install() -> Result<()> {
	let home_dir = utils::get_home_dir()?;

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

	let exe_dir = bin_dir.join("argon");

	if !exe_dir.exists() {
		let current_dir = env::current_exe()?;

		let remove_exe = logger::prompt("Installation completed! Do you want to remove this executable?", true);

		if remove_exe {
			fs::rename(&current_dir, &exe_dir)?;
		} else {
			fs::copy(&current_dir, &exe_dir)?;
		}
	}

	globenv::set_path(&bin_dir.to_str().unwrap())?;

	let default_template_dir = templates_dir.join("default");
	let default_project_dir = default_template_dir.join(".argon.project.json");
	let default_gitignore_dir = default_template_dir.join(".gitignore");
	let default_readme_dir = default_template_dir.join("README.md");

	if !default_template_dir.exists() {
		fs::create_dir(&default_template_dir)?;
	}

	if !default_project_dir.exists() {
		fs::write(default_project_dir, DEFAULT_PROJECT)?;
	}

	if !default_gitignore_dir.exists() {
		fs::write(default_gitignore_dir, DEFAULT_GITIGNORE)?;
	}

	if !default_readme_dir.exists() {
		fs::write(default_readme_dir, DEFAULT_README)?;
	}

	let compact_template_dir = templates_dir.join("compact");
	let compact_project_dir = compact_template_dir.join(".argon.project.json");

	if !compact_template_dir.exists() {
		fs::create_dir(&compact_template_dir)?;
	}

	if !compact_project_dir.exists() {
		fs::write(compact_project_dir, COMPACT_PROJECT)?;
	}

	Ok(())
}
