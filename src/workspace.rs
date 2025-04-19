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
pub struct WorkspaceLicense<'a> {
	pub inner: &'a str,
	pub force: bool,
}

#[derive(Debug)]
pub struct WorkspaceConfig<'a> {
	pub project: &'a Path,
	pub template: &'a str,
	pub license: WorkspaceLicense<'a>,
	pub git: bool,
	pub wally: bool,
	pub selene: bool,
	pub docs: bool,
	pub rojo_mode: bool,
	pub use_lua: bool,
}

pub fn init(workspace: WorkspaceConfig) -> Result<()> {
	let template_dir = util::get_argon_dir()?.join("templates").join(workspace.template);

	if !template_dir.exists() {
		bail!("Template {} does not exist", workspace.template.bold())
	}

	let workspace_dir = workspace.project.get_parent();
	let project_name = workspace_dir.get_name();

	if !workspace_dir.exists() {
		fs::create_dir_all(workspace_dir)?;
	}

	for entry in fs::read_dir(template_dir)? {
		let entry = entry?;

		let path = entry.path();
		let name = path.get_name();

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
				let contents = fs::read_to_string(path)?;
				let contents = contents.replace("$name", project_name);

				if workspace.wally {
					fs::write(new_path, contents)?;
				} else {
					let mut new_contents = String::new();
					let mut iterator = contents.lines();

					while let Some(line) = iterator.next() {
						if line.contains("Packages") {
							new_contents.pop();
							new_contents.pop();
							new_contents.push('\n');

							iterator.nth(1);
						} else {
							new_contents.push_str(&(line.to_owned() + "\n"));
						}
					}

					fs::write(new_path, new_contents)?;
				}
			}
			".gitignore" | ".github" => {
				if workspace.git {
					fs::copy(path, new_path)?;
				}
			}
			"wally.toml" => {
				if workspace.wally || workspace.template == "package" {
					let contents = fs::read_to_string(path)?;
					let contents = contents.replace("$name", &project_name.to_lowercase());
					let contents = contents.replace("$author", &util::get_username().to_lowercase());
					let contents = contents.replace("$license", workspace.license.inner);

					fs::write(new_path, contents)?;
				}
			}
			"selene.toml" => {
				if workspace.selene {
					fs::copy(path, new_path)?;
				}
			}
			_ => match path.get_stem() {
				"README" | "CHANGELOG" => {
					if workspace.docs {
						let contents = fs::read_to_string(path)?;
						let contents = contents.replace("$name", project_name);

						fs::write(new_path, contents)?;
					}
				}
				"LICENSE" => {
					if workspace.docs || workspace.license.force {
						let fallback = fs::read_to_string(path)?;
						add_license(&new_path, workspace.license.inner, &fallback)?;
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

	let template_dir = util::get_argon_dir()?.join("templates").join(template);

	if !template_dir.exists() {
		argon_warn!(
			"Template {} does not exist, additional files won't be added!",
			template.bold()
		);

		return Ok(Some(project));
	}

	let project_name = project.get_name();

	for entry in fs::read_dir(template_dir)? {
		let entry = entry?;

		let path = entry.path();
		let new_path = project.join(path.get_name());

		if new_path.exists() {
			continue;
		}

		match path.get_stem() {
			"wally" => {
				if workspace.wally || template == "package" {
					let contents = fs::read_to_string(path)?;
					let contents = contents.replace("$name", &project_name.to_lowercase());
					let contents = contents.replace("$author", &util::get_username().to_lowercase());

					fs::write(new_path, contents)?;
				}
			}
			"README" | "CHANGELOG" => {
				if workspace.docs {
					let contents = fs::read_to_string(path)?;
					let contents = contents.replace("$name", project_name);

					fs::write(new_path, contents)?;
				}
			}
			"LICENSE" => {
				if workspace.docs || workspace.license.force {
					let fallback = fs::read_to_string(path)?;
					add_license(&new_path, workspace.license.inner, &fallback)?;
				}
			}

			_ => {}
		}
	}

	Ok(Some(project))
}

pub fn initialize_repo(directory: &Path) -> Result<()> {
	let output = Program::new(ProgramName::Git)
		.message("Failed to initialize repository")
		.arg("init")
		.arg(directory.to_string())
		.output()?;

	if output.is_some() {
		debug!("Initialized Git repository");
	}

	Ok(())
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
					bail!("Bad SPDX License identifier")
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
			fs::copy(&path, to.join(name))?;
		}
	}

	Ok(())
}
