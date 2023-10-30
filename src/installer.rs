use directories::UserDirs;
use globenv;
use std::{env, error::Error, fs};

use crate::{confirm::prompt, unwrap_or_return};

pub fn install() -> Result<(), Box<dyn Error>> {
	let user_dirs = unwrap_or_return!(UserDirs::new(), Err("Failed to get user directory!".into()));
	let home_dir = user_dirs.home_dir();

	let argon_dir = home_dir.join(".argon");
	let bin_dir = argon_dir.join("bin");
	let exe_dir = bin_dir.join("argon");

	if !argon_dir.exists() {
		fs::create_dir(&argon_dir)?;
	}

	if !bin_dir.exists() {
		fs::create_dir(&bin_dir)?;
	}

	if !exe_dir.exists() {
		let current_dir = env::current_exe()?;

		let remove_exe = prompt("Installation completed! Do you want to remove this executable?", true)?;
		if remove_exe {
			fs::rename(&current_dir, &exe_dir)?;
		} else {
			fs::copy(&current_dir, &exe_dir)?;
		}
	}

	globenv::set_path(&bin_dir.to_str().unwrap())?;

	Ok(())
}
