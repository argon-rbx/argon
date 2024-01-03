use anyhow::Result;
use clap::{ArgAction, Parser};
use colored::Colorize;
use std::path::PathBuf;

use crate::{argon_info, argon_warn, config::Config, project, workspace};

/// Initialize new Argon project
#[derive(Parser)]
pub struct Init {
	/// Project path
	#[arg()]
	project: Option<PathBuf>,

	/// Workspace template
	#[arg(short = 'T', long)]
	template: Option<String>,

	/// Source folder
	#[arg(short, long)]
	source: Option<String>,

	/// Whether to configure Git
	#[arg(short, long, action = ArgAction::SetTrue)]
	git: bool,

	/// Whether to include docs (README, LICENSE, etc.)
	#[arg(short, long, action = ArgAction::SetTrue)]
	docs: bool,

	/// Whether to initialize using roblox-ts
	#[arg(short, long, action = ArgAction::SetTrue)]
	ts: bool,
}

impl Init {
	pub fn main(self) -> Result<()> {
		let config = Config::load();

		let project = self.project.unwrap_or_default();
		let template = self.template.unwrap_or(config.template);
		let source = self.source.unwrap_or(config.source_dir);
		let git = self.git || config.use_git;
		let docs = self.docs || config.include_docs;

		if self.ts {
			if workspace::init_ts(&project, &template, git)? {
				argon_info!("Successfully initialized roblox-ts project!");
			}

			return Ok(());
		}

		let project_path = project::resolve(project, &config.project_name)?;
		let project_exists = project_path.exists();

		if project_exists {
			argon_warn!("Project {} already exists!", project_path.to_str().unwrap().bold());
			return Ok(());
		}

		workspace::init(&project_path, &template, &source, git, docs)?;

		if git {
			let workspace_dir = workspace::get_dir(&project_path);

			workspace::initialize_repo(&workspace_dir)?;
		}

		argon_info!(
			"Successfully initialized project: {}",
			project_path.to_str().unwrap().bold()
		);

		Ok(())
	}
}
