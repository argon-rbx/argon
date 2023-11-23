use anyhow::{Context, Result};
use directories::UserDirs;
use std::{env, path::PathBuf};

pub fn get_home_dir() -> Result<PathBuf> {
	let user_dirs = UserDirs::new().context("Failed to get user directory")?;
	let home_dir = user_dirs.home_dir().to_path_buf();

	Ok(home_dir)
}

pub fn resolve_path(mut path: PathBuf) -> Result<PathBuf> {
	if path.is_absolute() {
		return Ok(path);
	}

	let current_dir = env::current_dir()?;
	path = current_dir.join(&path);

	Ok(path)
}

pub fn get_workspace_dir(project_path: PathBuf) -> PathBuf {
	let mut workspace_dir = project_path.to_owned();
	workspace_dir.pop();

	workspace_dir
}
