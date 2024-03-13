#![allow(clippy::new_without_default)]

pub mod cli;
pub mod config;
pub mod core;
pub mod crash_handler;
pub mod ext;
pub mod glob;
pub mod installer;
pub mod logger;
pub mod messages;
pub mod middleware;
pub mod program;
pub mod project;
pub mod resolution;
pub mod server;
pub mod sessions;
pub mod util;
pub mod vfs;
pub mod workspace;

// Paths that should be ignored before they are even processed
// useful to save ton of computing time, however users won't
// be able to set them in `sync_rules` or project `$path`
const BLACKLISTED_PATHS: [&str; 1] = [".DS_Store"];

/// A shorter way to lock the Mutex
#[macro_export]
macro_rules! lock {
	($mutex:expr) => {
		$mutex.lock().expect("Tried to lock Mutex that panicked!")
	};
}
