use globenv;
use std::{env, error::Error, fs};

use crate::{confirm::prompt, utils::get_home_dir};

pub fn install() -> Result<(), Box<dyn Error>> {
	let home_dir = get_home_dir()?;

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
