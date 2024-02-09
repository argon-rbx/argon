use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use log::{debug, info};
use roblox_install::RobloxStudio;
use std::{fs, path::PathBuf, process};

use crate::{
	argon_info,
	config::Config,
	core::Core,
	exit,
	program::{Program, ProgramKind},
	project::{self, Project},
	sessions,
	util::{self, PathExt},
};

/// Build project into Roblox place or model
#[derive(Parser)]
pub struct Build {
	/// Project path
	#[arg()]
	project: Option<PathBuf>,

	/// Session indentifier
	#[arg()]
	session: Option<String>,

	/// Output path
	#[arg(short, long)]
	output: Option<PathBuf>,

	/// Rebuild project every time files change
	#[arg(short, long)]
	watch: bool,

	/// Generate sourcemap every time files change
	#[arg(short, long)]
	sourcemap: bool,

	/// Build plugin and place it into plugins folder
	#[arg(short, long)]
	plugin: bool,

	/// Whether to build in XML format (.rbxlx or .rbxmx)
	#[arg(short, long)]
	xml: bool,

	/// Whether to build using roblox-ts
	#[arg(short, long)]
	ts: bool,

	/// Spawn the Argon child process
	#[arg(long, hide = true)]
	argon_spawn: bool,
}

impl Build {
	pub fn main(self) -> Result<()> {
		let config = Config::load();

		if self.watch && !self.argon_spawn && config.spawn {
			return self.spawn();
		}

		let project_path = project::resolve(self.project.clone().unwrap_or_default())?;
		let sourcemap_path = {
			if self.sourcemap {
				Some(project_path.get_parent().join("sourcemap.json"))
			} else {
				None
			}
		};

		if !project_path.exists() {
			exit!(
				"No project files found in {}",
				project_path.get_parent().to_string().bold()
			);
		}

		let project = Project::load(&project_path)?;

		let mut xml = self.xml;
		let path = if self.plugin {
			if project.is_place() {
				exit!("Cannot build plugin from place project");
			}

			let plugins_path = RobloxStudio::locate()?.plugins_path().to_owned();
			let ext = if xml { "rbxmx" } else { "rbxm" };

			plugins_path.join(format!("{}.{}", project.name, ext))
		} else if let Some(path) = self.output.clone() {
			if path.is_dir() {
				path.join(self.get_default_file(&project))
			} else {
				let ext = path.get_file_ext();

				if ext.is_empty() {
					fs::create_dir_all(&path)?;

					path.join(self.get_default_file(&project))
				} else {
					if ext == "rbxlx" || ext == "rbxmx" {
						xml = true;
					} else if ext == "rbxl" || ext == "rbxm" {
						xml = false;
					} else {
						exit!(
							"Invalid file extension: {}. Only {}, {}, {}, {} extensions are allowed.",
							ext,
							"rbxl".bold(),
							"rbxlx".bold(),
							"rbxm".bold(),
							"rbxmx".bold(),
						);
					}

					if ext.starts_with("rbxm") && project.is_place() {
						exit!("Cannot build model or plugin from place project");
					} else if ext.starts_with("rbxl") && !project.is_place() {
						exit!("Cannot build place from plugin or model project");
					}

					let parent = path.get_parent();

					if !parent.exists() {
						fs::create_dir_all(parent)?;
					}

					path
				}
			}
		} else {
			self.get_default_file(&project)
		};
		let path = path.resolve()?;

		let use_ts = self.ts || config.ts_mode || if config.auto_detect { project.is_ts() } else { false };

		if use_ts {
			argon_info!("Compiling TypeScript files..");

			let working_dir = project_path.get_parent();

			let child = Program::new(ProgramKind::Npx)
				.message("Failed to start roblox-ts")
				.current_dir(working_dir)
				.arg("rbxtsc")
				.arg("build")
				.spawn()?;

			if let Some(mut child) = child {
				child.wait()?;
			} else {
				return Ok(());
			}
		}

		let core = Core::new(project, self.watch)?;

		core.build(&path, xml)?;

		argon_info!(
			"Successfully built project: {} to: {}",
			project_path.to_string().bold(),
			path.to_string().bold()
		);

		if let Some(path) = &sourcemap_path {
			core.sourcemap(Some(path.clone()), false)?;

			argon_info!("Generated sourcemap in: {}", path.to_string().bold());
		}

		if self.watch {
			if use_ts {
				debug!("Starting roblox-ts");

				let working_dir = project_path.get_parent();

				let mut child = Program::new(ProgramKind::Npx)
					.current_dir(working_dir)
					.arg("rbxtsc")
					.arg("--watch")
					.spawn()?
					.unwrap();

				util::handle_kill(move || {
					child.kill().ok();
				})?;
			}

			if config.spawn {
				sessions::add(self.session, None, None, process::id())?;
			}

			argon_info!("Watching for changes..");

			for path_changed in core.tree_changed() {
				info!("Rebuilding project..");

				core.build(&path, xml)?;

				if path_changed {
					if let Some(path) = &sourcemap_path {
						info!("Regenerating sourcemap..");

						core.sourcemap(Some(path.clone()), false)?;
					}
				}
			}
		}

		Ok(())
	}

	fn get_default_file(&self, project: &Project) -> PathBuf {
		let ext = if project.is_place() {
			if self.xml {
				"rbxlx"
			} else {
				"rbxl"
			}
		} else if self.xml {
			"rbxmx"
		} else {
			"rbxm"
		};

		PathBuf::from(format!("{}.{}", project.name, ext))
	}

	fn spawn(self) -> Result<()> {
		let mut args = vec![String::from("build")];

		if let Some(project) = self.project {
			args.push(project.to_string())
		}

		if let Some(output) = self.output {
			args.push(output.to_string())
		}

		if self.watch {
			args.push(String::from("--watch"))
		}

		if self.sourcemap {
			args.push(String::from("--sourcemap"))
		}

		if self.plugin {
			args.push(String::from("--plugin"))
		}

		if self.xml {
			args.push(String::from("--xml"))
		}

		if self.ts {
			args.push(String::from("--ts"))
		}

		Program::new(ProgramKind::Argon).args(args).spawn()?;

		Ok(())
	}
}
