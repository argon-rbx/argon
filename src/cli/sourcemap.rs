use anyhow::{bail, Result};
use clap::Parser;
use colored::Colorize;
use log::info;
use std::{path::PathBuf, process};

use crate::{
	argon_info,
	config::Config,
	core::Core,
	ext::PathExt,
	program::{Program, ProgramName},
	project::{self, Project},
	sessions,
};

/// Generate JSON sourcemap of the project
#[derive(Parser)]
pub struct Sourcemap {
	/// Project path
	#[arg()]
	project: Option<PathBuf>,

	/// Session identifier
	#[arg()]
	session: Option<String>,

	/// Output path
	#[arg(short, long)]
	output: Option<PathBuf>,

	/// Regenerate sourcemap every time files change
	#[arg(short, long)]
	watch: bool,

	/// Whether non-script files should be included
	#[arg(short, long)]
	non_scripts: bool,

	/// Run Argon asynchronously
	#[arg(short = 'A', long = "async")]
	run_async: bool,

	/// Spawn the Argon child process (internal)
	#[arg(long, hide = true)]
	argon_spawn: bool,
}

impl Sourcemap {
	pub fn main(mut self) -> Result<()> {
		let project_path = project::resolve(self.project.clone().unwrap_or_default())?;

		Config::load_workspace(project_path.get_parent());
		let config = Config::new();

		if self.watch && !self.argon_spawn && (self.run_async || config.run_async) {
			return self.spawn();
		}

		if !project_path.exists() {
			bail!(
				"No project files found in {}",
				project_path.get_parent().to_string().bold()
			);
		}

		if let Some(path) = self.output.as_ref() {
			if config.smart_paths && (path.is_dir() || path.extension().is_none()) {
				let output = if path.get_name().to_lowercase() == "sourcemap" {
					path.with_extension("json")
				} else {
					path.join("sourcemap.json")
				};

				self.output = Some(output);
			}
		}

		let project = Project::load(&project_path)?;
		let core = Core::new(project, self.watch)?;

		core.sourcemap(self.output.clone(), self.non_scripts)?;

		if let Some(output) = &self.output {
			argon_info!(
				"Generated sourcemap of project: {} at: {}",
				project_path.to_string().bold(),
				output.resolve()?.to_string().bold()
			);
		}

		if self.watch {
			sessions::add(self.session, None, None, process::id(), config.run_async)?;

			if self.output.is_some() {
				argon_info!("Watching for changes..");
			}

			let queue = core.queue();
			queue.subscribe_internal().unwrap();

			loop {
				let _message = queue.get_change(0).unwrap();

				info!("Regenerating sourcemap..");
				core.sourcemap(self.output.clone(), self.non_scripts)?;
			}
		}

		Ok(())
	}

	fn spawn(self) -> Result<()> {
		let mut args = vec![String::from("sourcemap")];

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

		if self.non_scripts {
			args.push("--non-scripts".into())
		}

		Program::new(ProgramName::Argon).args(args).spawn()?;

		Ok(())
	}
}
