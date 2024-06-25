use anyhow::{bail, Result};
use chrono::Datelike;
use colored::Colorize;
use log::{debug, trace};
use reqwest::{blocking::Client, header::USER_AGENT};
use std::{
	fs,
	path::{Path, PathBuf},
};

use crate::{
	argon_info, argon_warn,
	config::Config,
	ext::PathExt,
	program::{Program, ProgramName},
	util,
};

#[derive(Debug)]
pub struct WorkspaceConfig<'a> {
	pub project: &'a Path,
	pub template: &'a str,
	pub license: &'a str,
	pub git: bool,
	pub wally: bool,
	pub docs: bool,
	pub rojo_mode: bool,
	pub use_lua: bool,
}

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

pub fn init(workspace: WorkspaceConfig) -> Result<()> {
	let templates_dir = util::get_argon_dir()?.join("templates").join(workspace.template);

	if !templates_dir.exists() {
		bail!("Template {} does not exist", workspace.template.bold())
	}

	let project_name = get_name(workspace.project);
	let workspace_dir = get_dir(workspace.project);

	if !workspace_dir.exists() {
		fs::create_dir_all(workspace_dir)?;
	}

	for entry in fs::read_dir(templates_dir)? {
		let entry = entry?;

		let path = entry.path();
		let name = path.get_name();
		let stem = path.get_stem();

		let new_path = if name == "project.json" {
			workspace.project.to_owned()
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
				if workspace.git {
					fs::copy(path, new_path)?;
				}
			}
			"wally.toml" => {
				if workspace.wally || workspace.template == "package" {
					let content = fs::read_to_string(path)?;
					let content = content.replace("$name", project_name);
					let content = content.replace("$author", &util::get_username());
					let content = content.replace("$license", workspace.license);

					fs::write(new_path, content)?;
				}
			}
			_ => match stem {
				"README" | "CHANGELOG" => {
					if workspace.docs {
						let content = fs::read_to_string(path)?;
						let content = content.replace("$name", project_name);

						fs::write(new_path, content)?;
					}
				}
				"LICENSE" => {
					let fallback = fs::read_to_string(path)?;

					if workspace.docs {
						add_license(&new_path, workspace.license, &fallback)?;
					}
				}
				_ => {
					if path.is_dir() {
						copy_dir(&path, &new_path, workspace.rojo_mode, workspace.use_lua)?;
					} else {
						fs::copy(path, new_path)?;
					}
				}
			},
		}
	}

	if workspace.git {
		initialize_repo(workspace_dir)?;
	}

	Ok(())
}

pub fn init_ts(workspace: WorkspaceConfig) -> Result<Option<PathBuf>> {
	let package_manager = &Config::new().package_manager;

	argon_info!("Waiting for {}..", package_manager.bold());

	let template = workspace.template;
	let mut project = workspace.project.to_owned();

	let env_yes = util::env_yes();
	let mut command = match template {
		"place" => "game",
		"plugin" => template,
		"package" => template,
		"model" => template,
		_ => "init",
	};

	if project.get_name().ends_with(".project.json") {
		project = workspace
			.project
			.parent()
			.map_or(PathBuf::new(), |path| path.to_owned());
	}

	if command == "init" && env_yes {
		command = "game";
	}

	let child = Program::new(ProgramName::Npm)
		.message("Failed to initialize roblox-ts project")
		.arg("create")
		.arg("roblox-ts")
		.arg(command)
		.arg("--skipBuild")
		.arg(format!("--git={}", workspace.git))
		.arg(format!("--packageManager={}", package_manager))
		.args(["--dir", &project.to_string()])
		.arg(if env_yes { "--yes" } else { "" })
		.spawn()?;

	if let Some(child) = child {
		let output = child.wait_with_output()?;

		if let Some(code) = output.status.code() {
			if code != 0 {
				return Ok(None);
			}
		} else {
			return Ok(None);
		}
	} else {
		return Ok(None);
	}

	if workspace.docs {
		let templates_dir = util::get_argon_dir()?.join("templates").join(template);

		if !templates_dir.exists() {
			argon_warn!("Template {} does not exist, docs won't be added!", template.bold());

			return Ok(Some(project));
		}

		let project_name = project.get_name();

		for entry in fs::read_dir(templates_dir)? {
			let entry = entry?;

			let path = entry.path();
			let name = path.get_name();
			let stem = path.get_stem();

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

					add_license(&new_path, workspace.license, &fallback)?;
				}
				"wally" => {
					if workspace.wally {
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

	Ok(Some(project))
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
	project_path.get_parent().get_name()
}

fn copy_dir(from: &Path, to: &Path, rojo_mode: bool, use_lua: bool) -> Result<()> {
	if !to.exists() {
		fs::create_dir_all(to)?;
	}

	for entry in fs::read_dir(from)? {
		let entry = entry?;

		let path = entry.path();
		let mut name = path.get_name().to_owned();

		if name.starts_with(".src") && rojo_mode {
			name = name.replace(".src", "init");
		}

		if name.ends_with(".luau") && use_lua {
			name = name.replace(".luau", ".lua");
		}

		if path.is_dir() {
			copy_dir(&path, &to.join(name), rojo_mode, use_lua)?;
		} else if name != ".gitkeep" {
			fs::copy(&path, &to.join(name))?;
		}
	}

	Ok(())
}
