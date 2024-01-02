use anyhow::{bail, Result};
use colored::Colorize;
use log::trace;
use std::{
	fs, io,
	path::{Path, PathBuf},
	process::Command,
};

use crate::{argon_error, argon_info, logger, util};

pub fn init(project: &Path, template: &String, source: &String) -> Result<()> {
	let home_dir = util::get_home_dir()?;
	let template_dir = home_dir.join(".argon").join("templates").join(template);

	let project_name = get_name(project);
	let workspace_dir = get_dir(project);

	if !workspace_dir.exists() {
		fs::create_dir_all(&workspace_dir)?;
	}

	for entry in fs::read_dir(template_dir)? {
		let entry = entry?;

		let file_path = entry.path();
		let file_name = entry.file_name();
		let file_name = file_name.to_str().unwrap();

		let new_file_path = if file_name == "project.json" {
			project.to_owned()
		} else {
			workspace_dir.join(file_name)
		};

		if new_file_path.exists() {
			continue;
		}

		if file_name == "project.json" || file_name == "README.md" {
			let content = fs::read_to_string(file_path)?;
			let content = content.replace("$name", &project_name);
			let content = content.replace("$source", source);

			fs::write(new_file_path, content)?;
		} else {
			fs::copy(file_path, new_file_path)?;
		}
	}

	let source_dir = workspace_dir.join(source);

	if !source_dir.exists() {
		fs::create_dir(source_dir)?;
	}

	Ok(())
}

pub fn init_ts(path: &Path) -> Result<bool> {
	argon_info!("Waiting for npm...");

	#[cfg(target_os = "windows")]
	let npm = "npm.cmd";
	#[cfg(not(target_os = "windows"))]
	let npm = "npm";

	let child = Command::new(npm)
		.arg("init")
		.arg("roblox-ts")
		.arg("--")
		.arg("--skipBuild")
		.args(["--dir", &util::path_to_string(path)])
		.spawn();

	match child {
		Ok(child) => {
			trace!("Initializing roblox-ts project");

			match child.wait_with_output() {
				Ok(output) => {
					if let Some(code) = output.status.code() {
						return Ok(code == 0);
					}

					Ok(false)
				}
				Err(err) => {
					bail!("Failed to initialize roblox-ts project: {}", err)
				}
			}
		}
		Err(err) => {
			if err.kind() == io::ErrorKind::NotFound {
				argon_error!("Failed to initialize roblox-ts project: npm is not installed");

				let install_npm = logger::prompt("Do you want to install npm now?", false);

				if install_npm {
					open::that("https://nodejs.org/en/download/")?;
				}

				Ok(false)
			} else {
				bail!("Failed to initialize roblox-ts project: {}", err)
			}
		}
	}
}

pub fn initialize_repo(directory: &PathBuf) -> Result<()> {
	match Command::new("git").arg("init").arg(directory).output() {
		Ok(_) => trace!("Initialized Git repository"),
		Err(err) => {
			if err.kind() == io::ErrorKind::NotFound {
				argon_error!(
					"Failed to initialize repository: Git is not installed. To suppress this message disable {} setting.",
					"git_init".bold()
				);

				let install_git = logger::prompt("Do you want to install Git now?", false);

				if install_git {
					open::that("https://git-scm.com/downloads")?;
				}
			} else {
				bail!("Failed to initialize Git repository: {}", err)
			}
		}
	}

	Ok(())
}

pub fn get_dir(project_path: &Path) -> PathBuf {
	let mut workspace_dir = project_path.to_owned();
	workspace_dir.pop();

	workspace_dir
}

pub fn get_name(project_path: &Path) -> String {
	let mut name = project_path.to_owned();
	name.pop();

	name.file_name().unwrap().to_str().unwrap().to_owned()
}
