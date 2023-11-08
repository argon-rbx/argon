use std::{env, path::PathBuf};

use anyhow::Result;

pub struct Project {}

pub fn resolve(mut project: PathBuf) -> Result<PathBuf> {
	if !project.is_absolute() {
		let current_dir = env::current_dir()?;
		project = current_dir.join(project);
	}

	Ok(project)
}
