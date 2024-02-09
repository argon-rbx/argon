use anyhow::{bail, Context, Result};
use directories::UserDirs;
use env_logger::WriteStyle;
use log::LevelFilter;
use rbx_reflection::ClassTag;
use std::{
	env,
	fmt::Display,
	path::{Path, PathBuf},
	process::{self, Command},
	sync::{Mutex, MutexGuard},
	thread,
	time::Duration,
};

/// More path methods ///

pub trait PathExt {
	fn resolve(&self) -> Result<PathBuf>;
	fn to_string(&self) -> String;
	fn get_file_name(&self) -> &str;
	fn get_file_stem(&self) -> &str;
	fn get_file_ext(&self) -> &str;
	fn get_parent(&self) -> &Path;
}

impl PathExt for Path {
	fn resolve(&self) -> Result<PathBuf> {
		if self.is_absolute() {
			return Ok(self.to_owned());
		}

		let current_dir = env::current_dir()?;
		let absolute = current_dir.join(self);

		Ok(absolute)
	}

	fn to_string(&self) -> String {
		self.to_str().unwrap_or_default().to_owned()
	}

	fn get_file_name(&self) -> &str {
		self.file_name().unwrap_or_default().to_str().unwrap_or_default()
	}

	fn get_file_stem(&self) -> &str {
		if !self.is_dir() {
			self.file_stem().unwrap_or_default().to_str().unwrap_or_default()
		} else {
			self.get_file_name()
		}
	}

	fn get_file_ext(&self) -> &str {
		if !self.is_dir() {
			self.extension().unwrap_or_default().to_str().unwrap_or_default()
		} else {
			""
		}
	}

	fn get_parent(&self) -> &Path {
		self.parent().unwrap_or(self)
	}
}

/// Additional method for `anyhow::Error` ///

pub trait Desc<T, E> {
	fn desc<D>(self, desc: D) -> Result<T, anyhow::Error>
	where
		D: Display + Send + Sync + 'static;

	fn with_desc<C, F>(self, f: F) -> Result<T, anyhow::Error>
	where
		C: Display + Send + Sync + 'static,
		F: FnOnce() -> C;
}

impl<T, E> Desc<T, E> for Result<T, E>
where
	E: Display + Send + Sync + 'static,
{
	fn desc<D>(self, desc: D) -> Result<T, anyhow::Error>
	where
		D: Display + Send + Sync + 'static,
	{
		match self {
			Ok(ok) => Ok(ok),
			Err(error) => {
				bail!("{}: {}", desc, error);
			}
		}
	}

	fn with_desc<C, F>(self, desc: F) -> Result<T, anyhow::Error>
	where
		C: Display + Send + Sync + 'static,
		F: FnOnce() -> C,
	{
		match self {
			Ok(ok) => Ok(ok),
			Err(error) => {
				bail!("{}: {}", desc(), error);
			}
		}
	}
}

/// `to_string` implementation for `WriteSetyle` ///

pub trait ToString {
	fn to_string(&self) -> String;
}

impl ToString for WriteStyle {
	fn to_string(&self) -> String {
		let write_style = match self {
			WriteStyle::Always => "always",
			WriteStyle::Auto => "auto",
			WriteStyle::Never => "never",
		};

		String::from(write_style)
	}
}

/// Returns the home directory of the current user
pub fn get_home_dir() -> Result<PathBuf> {
	let user_dirs = UserDirs::new().context("Failed to get user directory")?;
	let home_dir = user_dirs.home_dir().to_owned();

	Ok(home_dir)
}

/// Checks if the given `class` is a service
pub fn is_service(class: &str) -> bool {
	let descriptor = rbx_reflection_database::get().classes.get(class);

	let has_tag = if let Some(descriptor) = descriptor {
		descriptor.tags.contains(&ClassTag::Service)
	} else {
		false
	};

	has_tag || class == "StarterPlayerScripts" || class == "StarterCharacterScripts"
}

/// Checks if the given `class` is a script
pub fn is_script(class: &str) -> bool {
	class == "Script" || class == "LocalScript" || class == "ModuleScript"
}

/// Returns the Git or local username of the current user
pub fn get_username() -> String {
	if let Ok(output) = Command::new("git").arg("config").arg("user.name").output() {
		let username = String::from_utf8_lossy(&output.stdout).trim().to_owned();

		if !username.is_empty() {
			return username;
		}
	}

	whoami::username()
}

/// Kills the process with the given `pid`
pub fn kill_process(pid: u32) {
	#[cfg(not(target_os = "windows"))]
	Command::new("kill")
		.args(["-s", "INT"])
		.arg(pid.to_string())
		.output()
		.ok();

	#[cfg(target_os = "windows")]
	Command::new("taskkill")
		.arg("/T")
		.arg("/F")
		.args(["/pid", &pid.to_string()])
		.output()
		.ok();
}

/// Handles the kill signal
pub fn handle_kill<F>(mut handler: F) -> std::result::Result<(), ctrlc::Error>
where
	F: FnMut() + 'static + Send,
{
	ctrlc::set_handler(move || {
		handler();
		process::exit(0);
	})
}

/// Returns the `RUST_VERBOSE` environment variable
pub fn get_verbosity() -> LevelFilter {
	let verbosity = env::var("RUST_VERBOSE").unwrap_or("ERROR".to_owned());

	match verbosity.as_str() {
		"OFF" => LevelFilter::Off,
		"ERROR" => LevelFilter::Error,
		"WARN" => LevelFilter::Warn,
		"INFO" => LevelFilter::Info,
		"DEBUG" => LevelFilter::Debug,
		"TRACE" => LevelFilter::Trace,
		_ => LevelFilter::Error,
	}
}

/// Returns the `RUST_LOG_STYLE` environment variable
pub fn get_log_style() -> WriteStyle {
	let log_style = env::var("RUST_LOG_STYLE").unwrap_or("auto".to_owned());

	match log_style.as_str() {
		"always" => WriteStyle::Always,
		"never" => WriteStyle::Never,
		_ => WriteStyle::Auto,
	}
}

/// Returns the `RUST_BACKTRACE` environment variable
pub fn get_backtrace() -> bool {
	let backtrace = env::var("RUST_BACKTRACE").unwrap_or("0".to_owned());
	backtrace == "1"
}

/// Returns the `RUST_YES` environment variable
pub fn get_yes() -> bool {
	let yes = env::var("RUST_YES").unwrap_or("0".to_owned());
	yes == "1"
}

/// Waits for the Mutex to be released
pub fn wait_for_mutex<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
	loop {
		match mutex.try_lock() {
			Ok(guard) => {
				break guard;
			}
			Err(_) => {
				thread::sleep(Duration::from_millis(1));
			}
		}
	}
}
