use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use log::info;
use std::{path::PathBuf, process};

use crate::{
	argon_info,
	config::Config,
	core::Core,
	exit,
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

	/// Session indentifier
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

	/// Spawn the Argon child process
	#[arg(long, hide = true)]
	argon_spawn: bool,
}

impl Sourcemap {
	pub fn main(self) -> Result<()> {
		let config = Config::load();

		if self.watch && !self.argon_spawn && config.spawn {
			return self.spawn();
		}

		let project_path = project::resolve(self.project.clone().unwrap_or_default())?;

		if !project_path.exists() {
			exit!(
				"No project file found in {}",
				project_path.get_parent().to_string().bold()
			);
		}

		let project = Project::load(&project_path)?;
		let core = Core::new(project, self.watch)?;

		core.sourcemap(self.output.clone(), self.non_scripts)?;

		if let Some(output) = &self.output {
			argon_info!(
				"Successfully generated sourcemap of project: {} to: {}",
				project_path.to_string().bold(),
				output.to_string().bold()
			);
		}

		if self.watch {
			sessions::add(self.session, None, None, process::id(), config.spawn)?;

			if self.output.is_some() {
				argon_info!("Watching for changes..");
			}

			let queue = core.queue();
			queue.subscribe(1).unwrap();

			loop {
				let _message = queue.get(1).unwrap();

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

		if let Some(output) = self.output {
			args.push(output.to_string())
		}

		if self.watch {
			args.push(String::from("--watch"))
		}

		if self.non_scripts {
			args.push(String::from("--non-scripts"))
		}

		Program::new(ProgramName::Argon).args(args).spawn()?;

		Ok(())
	}
}
