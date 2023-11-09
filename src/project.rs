use anyhow::Result;
use std::{
	env,
	path::{PathBuf, MAIN_SEPARATOR},
};

pub struct Project {}

pub fn resolve(mut project: String, default: String) -> Result<PathBuf> {
	if project.ends_with(MAIN_SEPARATOR) {
		let mut project_glob = project.clone();
		project_glob.push_str("*.project.json");

		let mut project_path = PathBuf::from(project);
		let mut found_project = false;

		for path in glob::glob(&project_glob)? {
			project_path = path?;
			found_project = true;
			break;
		}

		if !found_project {
			let mut default_project = default;
			default_project.push_str(".project.json");

			project_path = project_path.join(default_project);
		}

		if !project_path.is_absolute() {
			let current_dir = env::current_dir()?;
			project_path = current_dir.join(project_path);
		}

		return Ok(project_path);
	}

	if !project.ends_with(".project.json") {
		project.push_str(".project.json")
	}

	let mut project_path = PathBuf::from(project);

	if !project_path.is_absolute() {
		let current_dir = env::current_dir()?;
		project_path = current_dir.join(project_path);
	}

	Ok(project_path)
}
