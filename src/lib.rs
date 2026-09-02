#![allow(clippy::new_without_default)]

use rbx_dom_weak::{types::Variant, UstrMap};

pub mod cli;
pub mod config;
pub mod constants;
pub mod core;
pub mod crash_handler;
pub mod ext;
pub mod glob;
pub mod installer;
pub mod integration;
pub mod logger;
pub mod middleware;
pub mod program;
pub mod project;
pub mod resolution;
pub mod server;
pub mod sessions;
pub mod stats;
pub mod studio;
pub mod updater;
pub mod util;
pub mod vfs;
pub mod workspace;

/// Global type for snapshot and instance properties
pub type Properties = UstrMap<Variant>;

/// A shorter way to lock the Mutex
#[macro_export]
macro_rules! lock {
	($mutex:expr) => {
		$mutex.lock().expect("Tried to lock Mutex that panicked!")
	};
}
