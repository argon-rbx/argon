use anyhow::{bail, Result};
use chrono::Datelike;
use colored::Colorize;
use log::{debug, trace};
use reqwest::{blocking::Client, header::USER_AGENT};
use std::{fs, path::Path};

use crate::{
	argon_info, argon_warn,
	ext::PathExt,
	program::{Program, ProgramName},
	util,
};

fn add_license(path: &Path, license: &str, fallback: &str) -> Result<()> {
	trace!("Getting {} license template..", license);

	let url = format!("https://api.github.com/licenses/{}", license);

	let license_template = || -> Result<String> {
		match Client::new().get(url).header(USER_AGENT, "Argon").send() {
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

	let name = util::get_username();
	let year = chrono::Utc::now().year();

	match license_template {
		Ok(license) => {
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
			let license = fallback.replace("$license", license);
			let license = license.replace("$year", &year.to_string());
			let license = license.replace("$owner", &name);

			fs::write(path, license)?;

			argon_warn!("Failed to add license: {}. Using basic fallback instead!", err);

			return Ok(());
		}
	}

	Ok(())
}

pub fn init(
	project: &Path,
	template: &str,
	license: &str,
	git: bool,
	wally: bool,
	docs: bool,
	rojo_mode: bool,
) -> Result<()> {
	let templates_dir = util::get_argon_dir()?.join("templates").join(template);

	if !templates_dir.exists() {
		bail!("Template {} does not exist", template)
	}

	let project_name = get_name(project);
	let workspace_dir = get_dir(project);

	if !workspace_dir.exists() {
		fs::create_dir_all(workspace_dir)?;
	}

	for entry in fs::read_dir(templates_dir)? {
		let entry = entry?;

		let path = entry.path();
		let name = path.get_file_name();
		let stem = path.get_file_stem();

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
					let fallback = fs::read_to_string(path)?;

					if docs {
						add_license(&new_path, license, &fallback)?;
					}
				}
				_ => {
					if path.is_dir() {
						copy_dir(&path, &new_path, rojo_mode)?;
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

	let child = Program::new(ProgramName::Npm)
		.message("Failed to initialize roblox-ts project")
		.arg("init")
		.arg("roblox-ts")
		.arg(command)
		.arg("--")
		.arg("--skipBuild")
		.arg(&format!("--git={}", git))
		.args(["--dir", &project.to_string()])
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
		return Ok(false);
	}

	if docs {
		let templates_dir = util::get_argon_dir()?.join("templates").join(template);

		if !templates_dir.exists() {
			argon_warn!("Template {} does not exist, docs won't be added!", template.bold());

			return Ok(true);
		}

		let project_name = project.get_file_name();

		for entry in fs::read_dir(templates_dir)? {
			let entry = entry?;

			let path = entry.path();
			let name = path.get_file_name();
			let stem = path.get_file_stem();

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
					let fallback = fs::read_to_string(path)?;

					add_license(&new_path, license, &fallback)?;
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
	let output = Program::new(ProgramName::Git)
		.message("Failed to initialize repository")
		.arg("init")
		.arg(&directory.to_string())
		.output()?;

	if output.is_some() {
		debug!("Initialized Git repository");
	}

	Ok(())
}

pub fn get_dir(project_path: &Path) -> &Path {
	project_path.get_parent()
}

pub fn get_name(project_path: &Path) -> &str {
	project_path.get_parent().get_file_name()
}

fn copy_dir(from: &Path, to: &Path, rojo_mode: bool) -> Result<()> {
	if !to.exists() {
		fs::create_dir_all(to)?;
	}

	for entry in fs::read_dir(from)? {
		let entry = entry?;

		let path = entry.path();
		let mut name = path.get_file_name().to_owned();

		if name.starts_with(".src") && rojo_mode {
			name = name.replace(".src", "init");
		}

		if path.is_dir() {
			copy_dir(&path, &to.join(name), rojo_mode)?;
		} else {
			fs::copy(&path, &to.join(name))?;
		}
	}

	Ok(())
}
