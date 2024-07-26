use anyhow::Result;
use colored::Colorize;
use log::debug;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

use crate::{
	logger,
	program::{Program, ProgramName},
};

#[derive(Debug, Deserialize)]
struct WallyManifest {
	dependencies: HashMap<String, String>,
}

fn install_wally_packages(workspace_path: &Path) -> Result<()> {
	let install = logger::prompt(
        &format!("Looks like your project uses Wally but Packages directory is missing or one of the dependencies is not installed. Would you like to run {} now?",
            "wally install".bold()
        ),
        true
    );

	if !install {
		return Ok(());
	}

	Program::new(ProgramName::Wally)
		.message("Failed to install dependencies")
		.arg("install")
		.current_dir(workspace_path)
		.output()?;

	Ok(())
}

pub fn check_wally_packages(workspace_path: &Path) -> Result<()> {
	let manifest_path = workspace_path.join("wally.toml");

	if !manifest_path.exists() {
		debug!("Aborted package verification: wally.toml does not exist");
		return Ok(());
	}

	let packages_path = workspace_path.join("Packages");
	let index_path = packages_path.join("_Index");

	if !packages_path.exists() || !index_path.exists() {
		return install_wally_packages(workspace_path);
	}

	let manifest: WallyManifest = toml::from_str(&fs::read_to_string(manifest_path)?)?;

	for (short, long) in manifest.dependencies {
		let short = short + ".lua";
		let long = long.replace('/', "_");

		if !packages_path.join(short).exists() || !index_path.join(long).exists() {
			return install_wally_packages(workspace_path);
		}
	}

	Ok(())
}
