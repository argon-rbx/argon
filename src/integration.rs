use anyhow::Result;
use colored::Colorize;
use log::{debug, warn};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

use crate::{
	glob::Glob,
	logger,
	program::{Program, ProgramName},
};

type Deps = HashMap<String, String>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct WallyManifest {
	dependencies: Option<Deps>,
	server_dependencies: Option<Deps>,
	dev_dependencies: Option<Deps>,
}

impl WallyManifest {
	fn get_directories(&self) -> HashMap<String, Deps> {
		let mut directories = HashMap::new();

		let mut join = |dependencies: &Option<Deps>, path: &str| {
			if let Some(dependency) = dependencies {
				if !dependency.is_empty() {
					directories.insert(path.to_owned(), dependency.clone());
				}
			}
		};

		join(&self.dependencies, "Packages");
		join(&self.server_dependencies, "ServerPackages");
		join(&self.dev_dependencies, "DevPackages");

		directories
	}
}

fn install_wally_packages(workspace_path: &Path) -> Result<()> {
	let install = logger::prompt(
        &format!("Looks like your project uses Wally but one of the directories is missing or one of the dependencies is not installed. Would you like to run {} now?",
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

pub fn check_wally_packages(workspace_path: &Path) {
	let manifest_path = workspace_path.join("wally.toml");

	if !manifest_path.exists() {
		debug!("Aborted package verification: wally.toml does not exist");
		return;
	}

	let verify_wally_manifest = || -> Result<()> {
		let manifest: WallyManifest = toml::from_str(&fs::read_to_string(manifest_path)?)?;

		for (path, dependencies) in manifest.get_directories() {
			let path = workspace_path.join(path);
			let index_path = path.join("_Index");

			if !path.exists() || !index_path.exists() {
				return install_wally_packages(workspace_path);
			}

			for (short, long) in dependencies {
				let short = Glob::from_path(&path.join(short + ".lua*"))?;

				let long = long.replace('/', "_");
				let long = long.rsplit_once("@").unwrap_or_default().0.to_owned();
				let long = Glob::from_path(&index_path.join(long + "*"))?;

				if short.first().is_none() || long.first().is_none() {
					return install_wally_packages(workspace_path);
				}
			}
		}

		Ok(())
	};

	match verify_wally_manifest() {
		Ok(()) => (),
		Err(err) => {
			warn!("Failed to verify or install missing Wally packages: {}", err);
		}
	}
}
