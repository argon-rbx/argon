use anyhow::{Context, Result};
use directories::UserDirs;
use std::{
	env,
	path::{Path, PathBuf},
};

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

pub fn get_file_extension(path: &Path) -> &str {
	path.extension().unwrap_or_default().to_str().unwrap_or_default()
}

pub fn get_file_name(path: &Path) -> &str {
	path.file_stem().unwrap().to_str().unwrap()
}

pub fn get_index<T: PartialEq>(vec: &[T], value: &T) -> Option<usize> {
	vec.iter().position(|v| v == value)
}
