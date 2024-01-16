use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

use crate::{argon_error, argon_info, config::Config, project, workspace};

/// Initialize new Argon project
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

	/// Include docs (README, LICENSE, etc.)
	#[arg(short, long)]
	docs: bool,

	/// Initialize using roblox-ts
	#[arg(short, long)]
	ts: bool,

	/// Initialize using Rojo namespace
	#[arg(short, long)]
	rojo: bool,
}

impl Init {
	pub fn main(self) -> Result<()> {
		let mut config = Config::load();

		let project = self.project.unwrap_or_default();
		let template = self.template.unwrap_or(config.template.clone());
		let license = self.license.unwrap_or(config.license.clone());
		let git = self.git || config.use_git;
		let docs = self.docs || config.include_docs;
		let ts = self.ts || config.ts_mode;
		let rojo = self.rojo || config.rojo_mode;

		if ts {
			if workspace::init_ts(&project, &template, &license, git, docs)? {
				argon_info!("Successfully initialized roblox-ts project!");
			}

			return Ok(());
		}

		if rojo {
			config.make_rojo();
		}

		let project_path = project::resolve(project, &config.project_name)?;

		if project_path.exists() {
			argon_error!("Project {} already exists!", project_path.to_str().unwrap().bold());
			return Ok(());
		}

		workspace::init(&project_path, &template, &license, git, docs)?;

		argon_info!(
			"Successfully initialized project: {}",
			project_path.to_str().unwrap().bold()
		);

		Ok(())
	}
}
