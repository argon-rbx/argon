use anyhow::Result;
use std::{env, fs};

use crate::{logger, util};

const DEFAULT_PROJECT: &str = include_str!("../assets/templates/default/project.json");
const DEFAULT_GITIGNORE: &str = include_str!("../assets/templates/default/.gitignore");
const DEFAULT_README: &str = include_str!("../assets/templates/default/README.md");

const COMPACT_PROJECT: &str = include_str!("../assets/templates/compact/project.json");

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

	let default_template_dir = templates_dir.join("default");
	let default_project_path = default_template_dir.join("project.json");
	let default_gitignore_path = default_template_dir.join(".gitignore");
	let default_readme_path = default_template_dir.join("README.md");

	if !default_template_dir.exists() {
		fs::create_dir(&default_template_dir)?;
	}

	if !default_project_path.exists() {
		fs::write(default_project_path, DEFAULT_PROJECT)?;
	}

	if !default_gitignore_path.exists() {
		fs::write(default_gitignore_path, DEFAULT_GITIGNORE)?;
	}

	if !default_readme_path.exists() {
		fs::write(default_readme_path, DEFAULT_README)?;
	}

	let compact_template_dir = templates_dir.join("compact");
	let compact_project_path = compact_template_dir.join("project.json");

	if !compact_template_dir.exists() {
		fs::create_dir(&compact_template_dir)?;
	}

	if !compact_project_path.exists() {
		fs::write(compact_project_path, COMPACT_PROJECT)?;
	}

	Ok(())
}
