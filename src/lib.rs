#![allow(clippy::new_without_default)]

pub mod cli;
pub mod config;
pub mod core;
pub mod crash_handler;
pub mod fs;
pub mod glob;
pub mod installer;
pub mod logger;
pub mod messages;
pub mod program;
pub mod project;
pub mod rbx_path;
pub mod resolution;
pub mod server;
pub mod sessions;
pub mod util;
pub mod workspace;

// A shorter way to lock the Mutex
#[macro_export]
macro_rules! lock {
	($mutex:expr) => {
		$mutex.lock().unwrap()
	};
}
