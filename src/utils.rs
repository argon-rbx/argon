use anyhow::{bail, Context, Result};
use colored::Colorize;
use directories::UserDirs;
use log::trace;
use std::{io, path::PathBuf, process::Command};

use crate::argon_error;

pub fn get_home_dir() -> Result<PathBuf> {
	let user_dirs = UserDirs::new().context("Failed to get user directory")?;
	let home_dir = user_dirs.home_dir().to_path_buf();

	Ok(home_dir)
}

pub fn initialize_repo(directory: &PathBuf) -> Result<()> {
	match Command::new("git").arg("init").arg(directory).output() {
		Ok(_) => trace!("Initialized Git repository"),
		Err(error) => {
			if error.kind() == io::ErrorKind::NotFound {
				argon_error!(
					"Failed to initialize repository: Git is not installed. To suppress this message disable {} setting.",
					"git_init".bold()
				);
			} else {
				bail!("Failed to initialize Git repository: {}", error)
			}
		}
	}

	Ok(())
}
