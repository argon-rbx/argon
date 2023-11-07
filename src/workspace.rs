use anyhow::Result;
use std::{env, path::PathBuf};

use crate::{argon_error, argon_warn, config::Config};

// TODO: make project.rs function
pub fn verify(mut project: PathBuf) -> Result<(PathBuf, bool)> {
	if !project.is_absolute() {
		let current_dir = env::current_dir()?;
		project = current_dir.join(project);
	}

	Ok((project.to_path_buf(), project.exists()))
}

pub fn init(project: &PathBuf, auto_init: bool) -> Result<()> {
	let (project, exists) = verify(project.to_owned())?;

	if !exists && auto_init {
		// TODO: load template
		argon_warn!("Cannot find the project, creating new one!")
	} else if !exists {
		argon_error!(
			"Project file does not exist in this directory: {}. Run `argon init` or enable `auto_init` setting.",
			project.to_str().unwrap()
		);

		// Err()
	}

	Ok(())
}
