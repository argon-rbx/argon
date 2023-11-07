use anyhow::{Context, Result};
use directories::UserDirs;
use std::path::PathBuf;

pub fn get_home_dir() -> Result<PathBuf> {
	let user_dirs = UserDirs::new().context("Failed to get user directory")?;
	let home_dir = user_dirs.home_dir().to_path_buf();

	Ok(home_dir)
}
