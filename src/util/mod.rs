use anyhow::{Context, Result};
use directories::UserDirs;
use rbx_reflection::ClassTag;
use std::{
	env,
	ffi::OsStr,
	path::{Path, PathBuf},
	process::{self, Command},
};

pub mod csv;
pub mod json;

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

pub fn path_to_string(path: &Path) -> String {
	path.to_str().unwrap_or_default().to_owned()
}

pub fn from_os_str(str: &OsStr) -> &str {
	str.to_str().unwrap_or_default()
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

pub fn is_service(class: &str) -> bool {
	let descriptor = rbx_reflection_database::get().classes.get(class);

	let has_tag = if let Some(descriptor) = descriptor {
		descriptor.tags.contains(&ClassTag::Service)
	} else {
		false
	};

	has_tag || class == "StarterPlayerScripts" || class == "StarterCharacterScripts"
}

pub fn get_username() -> String {
	if let Ok(output) = Command::new("git").arg("config").arg("user.name").output() {
		return String::from_utf8_lossy(&output.stdout).trim().to_string();
	}

	whoami::username()
}

pub fn kill(pid: u32) {
	#[cfg(not(target_os = "windows"))]
	{
		Command::new("kill")
			.args(["-s", "INT"])
			.arg(pid.to_string())
			.output()
			.ok();
	}

	// TODO: needs improvement
	#[cfg(target_os = "windows")]
	{
		Command::new("taskkill")
			.arg("/F")
			.args(["/pid", &pid.to_string()])
			.output()
			.ok();
	}
}

pub fn handle_kill<F>(mut handler: F) -> std::result::Result<(), ctrlc::Error>
where
	F: FnMut() + 'static + Send,
{
	ctrlc::set_handler(move || {
		handler();
		process::exit(0);
	})
}
