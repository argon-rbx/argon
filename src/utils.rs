use anyhow::{Context, Result};
use directories::UserDirs;
use std::{
	env,
	ffi::OsStr,
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

pub fn get_file_name(path: &Path) -> &str {
	path.file_name().unwrap().to_str().unwrap()
}

pub fn get_file_stem(path: &Path) -> &str {
	path.file_stem().unwrap().to_str().unwrap()
}

pub fn get_file_ext(path: &Path) -> &str {
	path.extension().unwrap_or_default().to_str().unwrap_or_default()
}

pub fn get_index<T: PartialEq>(slice: &[T], item: &T) -> Option<usize> {
	slice.iter().position(|i| i == item)
}

pub fn from_os_str(str: &OsStr) -> &str {
	str.to_str().unwrap_or_default()
}
