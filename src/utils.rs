use directories::UserDirs;
use std::path::PathBuf;

use crate::DynResult;

pub fn get_home_dir() -> DynResult<PathBuf> {
	let user_dirs = UserDirs::new().ok_or("Failed to get user directory!")?;
	let home_dir = user_dirs.home_dir().to_path_buf();

	Ok(home_dir)
}
