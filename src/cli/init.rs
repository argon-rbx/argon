use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

use crate::{argon_error, argon_info, config::Config, ext::PathExt, project, workspace};

/// Initialize a new Argon project
#[derive(Parser)]
pub struct Init {
	/// Project path
	#[arg()]
	project: Option<PathBuf>,

	/// Workspace template
	#[arg(short = 'T', long)]
	template: Option<String>,

	/// Workspace license
	#[arg(short, long)]
	license: Option<String>,

	/// Configure Git
	#[arg(short, long)]
	git: bool,

	/// Setup Wally
	#[arg(short, long)]
	wally: bool,

	/// Include docs (README, CHANGELOG, etc.)
	#[arg(short, long)]
	docs: bool,

	/// Initialize using roblox-ts
	#[arg(short, long)]
	ts: bool,
}

impl Init {
	pub fn main(self) -> Result<()> {
		let config = Config::load();

		let project = self.project.unwrap_or_default();
		let template = self.template.unwrap_or(config.template.clone());
		let license = self.license.unwrap_or(config.license.clone());
		let git = self.git || config.use_git;
		let wally = self.wally || config.use_wally;
		let docs = self.docs || config.include_docs;
		let ts = self.ts || config.ts_mode;

		if ts {
			if workspace::init_ts(&project, &template, &license, git, wally, docs)? {
				let path = project.resolve()?.join("default.project.json");

				argon_info!(
					"Successfully initialized roblox-ts project: {}",
					path.to_string().bold()
				);
			}

			return Ok(());
		}

		let project_path = project::resolve(project)?;

		if project_path.exists() {
			argon_error!("Project {} already exists!", project_path.to_string().bold());
			return Ok(());
		}

		workspace::init(&project_path, &template, &license, git, wally, docs, config.rojo_mode)?;

		argon_info!("Successfully initialized project: {}", project_path.to_string().bold());

		Ok(())
	}
}
