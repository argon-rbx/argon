#![allow(clippy::new_without_default)]

pub mod cli;
pub mod config;
pub mod core;
pub mod crash_handler;
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

/// A shorter way to lock the Mutex.
/// Will panic if the Mutex is already locked.
#[macro_export]
macro_rules! lock {
	($mutex:expr) => {
		$mutex.try_lock().expect("Tried to lock Mutex that is already locked!")
	};
}
