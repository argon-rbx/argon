use anyhow::{bail, Result};
use colored::Colorize;
use log::trace;
use std::{
	fs, io,
	path::{Path, PathBuf},
	process::Command,
};

use crate::{argon_error, logger, utils};

pub fn init(project: &Path, template: &String, source: &String) -> Result<()> {
	let home_dir = utils::get_home_dir()?;
	let template_dir = home_dir.join(".argon").join("templates").join(template);

	let project_name = project.file_name().unwrap().to_str().unwrap();
	let workspace_dir = get_dir(project.to_owned());

	if !workspace_dir.exists() {
		fs::create_dir_all(&workspace_dir)?;
	}

	for dir_entry in fs::read_dir(template_dir)? {
		let dir_entry = dir_entry?;

		let file_path = dir_entry.path();
		let file_name = dir_entry.file_name();
		let file_name = file_name.to_str().unwrap();

		let new_file_path = if file_name == "project.json" {
			workspace_dir.join(project_name)
		} else {
			workspace_dir.join(file_name)
		};

		if new_file_path.exists() {
			continue;
		}

		if file_name == "project.json" || file_name == "README.md" {
			let mut name = project_name.replace(".project.json", "");

			if name == ".argon" {
				name = String::from("Argon project");
			}

			let content = fs::read_to_string(file_path)?;
			let content = content.replace("$name", &name);
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

pub fn initialize_repo(directory: &PathBuf) -> Result<()> {
	match Command::new("git").arg("init").arg(directory).output() {
		Ok(_) => trace!("Initialized Git repository"),
		Err(error) => {
			if error.kind() == io::ErrorKind::NotFound {
				argon_error!(
					"Failed to initialize repository: Git is not installed. To suppress this message disable {} setting.",
					"git_init".bold()
				);

				let install_git = logger::prompt("Do you want to install Git now?", false);

				if install_git {
					open::that("https://git-scm.com/downloads")?;
				}
			} else {
				bail!("Failed to initialize Git repository: {}", error)
			}
		}
	}

	Ok(())
}

pub fn get_dir(project_path: PathBuf) -> PathBuf {
	let mut workspace_dir = project_path.to_owned();
	workspace_dir.pop();

	workspace_dir
}
