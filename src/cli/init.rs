use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::{argon_info, argon_warn, config::Config, project, workspace};

#[derive(Parser)]
pub struct Init {
	/// Project path [default: ".argon"]
	#[arg()]
	project: Option<String>,

	/// Workspace template [default: "default"]
	#[arg(short, long)]
	template: Option<String>,

	/// Source folder [default: "src"]
	#[arg(short, long)]
	source: Option<String>,
}

impl Init {
	pub fn main(self) -> Result<()> {
		let config = Config::load();

		let project = self.project.unwrap_or(config.project_name.clone());
		let template = self.template.unwrap_or(config.template);
		let source = self.source.unwrap_or(config.source_dir);

		let project_path = project::resolve(project, &config.project_name)?;
		let project_exists = project_path.exists();

		if project_exists {
			argon_warn!("Project {} already exists!", project_path.to_str().unwrap().bold());
			return Ok(());
		}

		workspace::init(&project_path, &template, &source)?;

		if config.git_init {
			let workspace_dir = workspace::get_dir(project_path.to_owned());

			workspace::initialize_repo(&workspace_dir)?;
		}

		argon_info!(
			"Successfully initialized project: {}",
			project_path.to_str().unwrap().bold()
		);

		Ok(())
	}
}
