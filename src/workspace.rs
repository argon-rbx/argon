use anyhow::Result;
use std::{fs, path::PathBuf};

use crate::utils;

pub fn init(project: &PathBuf, template: String) -> Result<()> {
	let home_dir = utils::get_home_dir()?;
	let template_dir = home_dir.join(".argon").join("templates").join(template);

	let project_name = project.file_name().unwrap().to_str().unwrap();
	let mut project_dir = project.to_path_buf();
	project_dir.pop();

	if !project_dir.exists() {
		fs::create_dir_all(&project_dir)?;
	}

	for dir_entry in fs::read_dir(template_dir)? {
		let dir_entry = dir_entry?;

		let file_path = dir_entry.path();
		let file_name = dir_entry.file_name();
		let file_name = file_name.to_str().unwrap();

		let new_file_path: PathBuf;

		if file_name == "project.json" {
			new_file_path = project_dir.join(project_name);
		} else {
			new_file_path = project_dir.join(file_name);
		}

		if new_file_path.exists() {
			continue;
		}

		if file_name == "project.json" || file_name == "README.md" {
			let name = project_name.replace(".project.json", "");

			let content = fs::read_to_string(file_path)?;
			let content = content.replace("$name", &name);

			fs::write(new_file_path, content)?;
		} else {
			fs::copy(file_path, new_file_path)?;
		}
	}

	fs::create_dir(project_dir.join("src"))?;

	Ok(())
}
