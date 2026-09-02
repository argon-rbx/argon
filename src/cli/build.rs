use anyhow::{bail, Result};
use clap::Parser;
use colored::Colorize;
use log::{debug, info};
use roblox_install::RobloxStudio;
use std::{fs, path::PathBuf, process};

use crate::{
	argon_info,
	config::Config,
	core::Core,
	ext::PathExt,
	integration,
	program::{Program, ProgramName},
	project::{self, Project},
	sessions,
};

/// Build project into Roblox binary or XML place or model
#[derive(Parser)]
pub struct Build {
	/// Project path
	#[arg()]
	project: Option<PathBuf>,

	/// Session identifier
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

	/// Run Argon asynchronously
	#[arg(short = 'A', long = "async")]
	run_async: bool,

	/// Spawn the Argon child process (internal)
	#[arg(long, hide = true)]
	argon_spawn: bool,
}

impl Build {
	pub fn main(self) -> Result<()> {
		let project_path = project::resolve(self.project.clone().unwrap_or_default())?;

		Config::load_workspace(project_path.get_parent());
		let config = Config::new();

		if self.watch && !self.argon_spawn && (self.run_async || config.run_async) {
			return self.spawn();
		}

		let sourcemap_path = if self.sourcemap || config.with_sourcemap {
			Some(project_path.with_file_name("sourcemap.json"))
		} else {
			None
		};

		if !project_path.exists() {
			bail!(
				"No project files found in {}",
				project_path.get_parent().to_string().bold()
			);
		}

		let project = Project::load(&project_path)?;

		let mut xml = self.xml || config.build_xml;
		let path = if self.plugin {
			if project.is_place() {
				bail!("Cannot build plugin from place project");
			}

			let plugins_path = RobloxStudio::locate()?.plugins_path().to_owned();
			let ext = if xml { "rbxmx" } else { "rbxm" };

			plugins_path.join(format!("{}.{}", project.name, ext))
		} else if let Some(path) = self.output.clone() {
			if path.is_dir() {
				path.join(self.get_default_file(&project))
			} else {
				let ext = path.get_ext();

				if ext.is_empty() && !config.smart_paths {
					fs::create_dir_all(&path)?;

					path.join(self.get_default_file(&project))
				} else {
					if ext == "rbxlx" || ext == "rbxmx" {
						xml = true;
					} else if ext == "rbxl" || ext == "rbxm" {
						xml = false;
					} else {
						bail!(
							"Invalid file extension: {}. Only {}, {}, {}, {} extensions are allowed",
							ext.bold(),
							"rbxl".bold(),
							"rbxlx".bold(),
							"rbxm".bold(),
							"rbxmx".bold(),
						);
					}

					if ext.starts_with("rbxm") && project.is_place() {
						bail!("Cannot build model or plugin from place project");
					} else if ext.starts_with("rbxl") && !project.is_place() {
						bail!("Cannot build place from plugin or model project");
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
		}
		.resolve()?;

		let use_wally = config.use_wally || (config.detect_project && project.is_wally());
		let use_ts = self.ts || config.ts_mode || (config.detect_project && project.is_ts());

		if use_wally {
			integration::check_wally_packages(&project.workspace_dir);
		}

		if use_ts {
			argon_info!("Compiling TypeScript files..");

			let working_dir = project_path.get_parent();

			let child = Program::new(ProgramName::Npx)
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

			argon_info!("Generated sourcemap at: {}", path.to_string().bold());
		}

		if self.watch {
			if use_ts {
				debug!("Starting roblox-ts");

				let working_dir = project_path.get_parent();

				Program::new(ProgramName::Npx)
					.current_dir(working_dir)
					.arg("rbxtsc")
					.arg("--watch")
					.spawn()?;
			}

			sessions::add(self.session, None, None, process::id(), config.run_async)?;

			argon_info!("Watching for changes..");

			let queue = core.queue();
			queue.subscribe_internal().unwrap();

			loop {
				let _message = queue.get_change(0).unwrap();

				info!("Rebuilding project..");
				core.build(&path, xml)?;

				if let Some(path) = &sourcemap_path {
					info!("Regenerating sourcemap..");
					core.sourcemap(Some(path.clone()), false)?;
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

		if let Some(session) = self.session {
			args.push(session);
		}

		if let Some(output) = self.output {
			args.push("--output".into());
			args.push(output.to_string())
		}

		if self.watch {
			args.push("--watch".into())
		}

		if self.sourcemap {
			args.push("--sourcemap".into())
		}

		if self.plugin {
			args.push("--plugin".into())
		}

		if self.xml {
			args.push("--xml".into())
		}

		if self.ts {
			args.push("--ts".into())
		}

		Program::new(ProgramName::Argon).args(args).spawn()?;

		Ok(())
	}
}
