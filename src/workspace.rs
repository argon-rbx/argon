use anyhow::{bail, Result};
use chrono::Datelike;
use log::trace;
use reqwest::blocking::Client;
use std::{fs, path::Path};

use crate::{
	argon_info, argon_warn,
	program::{Program, ProgramKind},
	util,
};

fn add_license(path: &Path, license: &str) -> Result<()> {
	let url = format!("https://api.github.com/licenses/{}", license);

	let license = || -> Result<String> {
		match Client::new().get(url).header("User-Agent", "Argon").send() {
			Ok(response) => {
				let json = response.json::<serde_json::Value>()?;

				if let Some(body) = json["body"].as_str() {
					Ok(body.to_owned())
				} else {
					bail!("Bad SPDX License ID")
				}
			}
			Err(_) => {
				bail!("No internet connection");
			}
		}
	}();

	match license {
		Ok(license) => {
			let name = util::get_username();
			let year = chrono::Utc::now().year();

			// Apache
			let license = license.replace("[yyyy]", &year.to_string());
			let license = license.replace("[name of copyright owner]", &name);

			// MIT & BSD
			let license = license.replace("[year]", &year.to_string());
			let license = license.replace("[fullname]", &name);

			// GNU
			let license = license.replace("<year>", &year.to_string());
			let license = license.replace("<name of author>", &name);

			fs::write(path, license)?;
		}
		Err(err) => {
			argon_warn!("Failed to add license: {}", err);
			return Ok(());
		}
	}

	Ok(())
}

pub fn init(project: &Path, template: &str, license: &str, git: bool, wally: bool, docs: bool) -> Result<()> {
	let home_dir = util::get_home_dir()?;
	let template_dir = home_dir.join(".argon").join("templates").join(template);

	if !template_dir.exists() {
		bail!("Template {} does not exist", template)
	}

	let project_name = get_name(project);
	let workspace_dir = get_dir(project);

	if !workspace_dir.exists() {
		fs::create_dir_all(workspace_dir)?;
	}

	for entry in fs::read_dir(template_dir)? {
		let entry = entry?;

		let path = entry.path();
		let name = util::get_file_name(&path);
		let stem = util::get_file_stem(&path);

		let new_path = if name == "project.json" {
			project.to_owned()
		} else {
			workspace_dir.join(name)
		};

		if new_path.exists() {
			continue;
		}

		match name {
			"project.json" => {
				let content = fs::read_to_string(path)?;
				let content = content.replace("$name", project_name);

				fs::write(new_path, content)?;
			}
			".gitignore" | ".github" => {
				if git {
					fs::copy(path, new_path)?;
				}
			}
			"wally.toml" => {
				if wally || template == "package" {
					let content = fs::read_to_string(path)?;
					let content = content.replace("$name", project_name);
					let content = content.replace("$author", &util::get_username());
					let content = content.replace("$license", license);

					fs::write(new_path, content)?;
				}
			}
			_ => match stem {
				"README" | "CHANGELOG" => {
					if docs {
						let content = fs::read_to_string(path)?;
						let content = content.replace("$name", project_name);

						fs::write(new_path, content)?;
					}
				}
				"LICENSE" => {
					if docs {
						add_license(&new_path, license)?;
					}
				}
				_ => {
					if path.is_dir() {
						util::copy_dir(&path, &new_path)?;
					} else {
						fs::copy(path, new_path)?;
					}
				}
			},
		}
	}

	if git {
		initialize_repo(workspace_dir)?;
	}

	Ok(())
}

pub fn init_ts(project: &Path, template: &str, license: &str, git: bool, wally: bool, docs: bool) -> Result<bool> {
	argon_info!("Waiting for npm..");

	let command = match template {
		"place" => "game",
		"plugin" => template,
		"package" => template,
		"model" => template,
		_ => "init",
	};

	let child = Program::new(ProgramKind::Npm)
		.message("Failed to initialize roblox-ts project")
		.arg("init")
		.arg("roblox-ts")
		.arg(command)
		.arg("--")
		.arg("--skipBuild")
		.arg(&format!("--git={}", git))
		.args(["--dir", &util::path_to_string(project)])
		.arg(if util::get_yes() { "--yes" } else { "" })
		.spawn()?;

	if let Some(child) = child {
		let output = child.wait_with_output()?;

		if let Some(code) = output.status.code() {
			if code != 0 {
				return Ok(false);
			}
		} else {
			return Ok(false);
		}
	} else {
		trace!("npm is not installed");
		return Ok(false);
	}

	if docs {
		let home_dir = util::get_home_dir()?;
		let template_dir = home_dir.join(".argon").join("templates").join(template);

		if !template_dir.exists() {
			argon_warn!("Template {} does not exist, docs won't be added!", template);

			return Ok(true);
		}

		let project_name = util::get_file_name(project);

		for entry in fs::read_dir(template_dir)? {
			let entry = entry?;

			let path = entry.path();
			let name = util::get_file_name(&path);
			let stem = util::get_file_stem(&path);

			let new_path = project.join(name);

			if new_path.exists() {
				continue;
			}

			match stem {
				"README" | "CHANGELOG" => {
					let content = fs::read_to_string(path)?;
					let content = content.replace("$name", project_name);

					fs::write(new_path, content)?;
				}
				"LICENSE" => {
					add_license(&new_path, license)?;
				}
				"wally" => {
					if wally {
						let content = fs::read_to_string(path)?;
						let content = content.replace("$name", project_name);
						let content = content.replace("$author", &util::get_username());

						fs::write(new_path, content)?;
					}
				}
				_ => {}
			}
		}
	}

	Ok(true)
}

pub fn initialize_repo(directory: &Path) -> Result<()> {
	let output = Program::new(ProgramKind::Git)
		.message("Failed to initialize repository")
		.arg("init")
		.arg(&util::path_to_string(directory))
		.output()?;

	if output.is_some() {
		trace!("Initialized Git repository");
	} else {
		trace!("Git is not installed");
	}

	Ok(())
}

pub fn get_dir(project_path: &Path) -> &Path {
	project_path.parent().unwrap()
}

pub fn get_name(project_path: &Path) -> &str {
	let name = project_path.parent().unwrap();

	util::from_os_str(name.file_name().unwrap())
}
