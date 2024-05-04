use anyhow::Result;
use clap::{ArgAction, Parser};
use colored::Colorize;
use std::path::PathBuf;

use crate::{argon_error, argon_info, config::Config, ext::PathExt, project, stats, workspace};

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
	#[arg(
		short,
        long,
        default_missing_value("true"),
        num_args(0..=1),
    	action = ArgAction::Set,
    )]
	git: Option<bool>,

	/// Setup Wally
	#[arg(
		short,
        long,
        default_missing_value("true"),
        num_args(0..=1),
    	action = ArgAction::Set,
    )]
	wally: Option<bool>,

	/// Include docs (README, CHANGELOG, etc.)
	#[arg(
		short,
        long,
        default_missing_value("true"),
        num_args(0..=1),
    	action = ArgAction::Set,
    )]
	docs: Option<bool>,

	/// Initialize using roblox-ts
	#[arg(
		short,
        long,
        default_missing_value("true"),
        num_args(0..=1),
    	action = ArgAction::Set,
    )]
	ts: Option<bool>,
}

impl Init {
	pub fn main(self) -> Result<()> {
		let config = Config::new();

		let project = self.project.unwrap_or_default();
		let template = self.template.unwrap_or(config.template.clone());
		let license = self.license.unwrap_or(config.license.clone());
		let git = self.git.unwrap_or(config.use_git);
		let wally = self.wally.unwrap_or(config.use_wally);
		let docs = self.docs.unwrap_or(config.include_docs);
		let ts = self.ts.unwrap_or(config.ts_mode);

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

		stats::projects_created(1);

		Ok(())
	}
}
