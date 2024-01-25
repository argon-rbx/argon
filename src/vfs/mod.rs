use anyhow::Result;
use crossbeam_channel::Receiver;
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use self::watcher::VfsWatcher;

pub mod debouncer;
pub mod watcher;

#[derive(Debug, Clone)]
pub enum VfsEvent {
	Create(PathBuf),
	Delete(PathBuf),
	Write(PathBuf),
}

pub struct Vfs {
	watcher: VfsWatcher,
	watch_map: HashMap<PathBuf, bool>,
}

impl Vfs {
	pub fn new() -> Result<Self> {
		Ok(Self {
			watcher: VfsWatcher::new()?,
			watch_map: HashMap::new(),
		})
	}

	pub fn watch(&mut self, path: &Path) -> Result<()> {
		self.watcher.watch(path)?;
		self.watch_map.insert(path.to_owned(), path.is_dir());

		Ok(())
	}

	pub fn unwatch(&mut self, path: &Path) -> Result<()> {
		self.watcher.unwatch(path)?;

		let mut unwatched = vec![];

		for path in self.watch_map.keys() {
			if path.starts_with(path) {
				self.watcher.unwatch(path)?;

				unwatched.push(path.to_owned());
			}
		}

		for path in unwatched {
			self.watch_map.remove(&path);
		}

		Ok(())
	}

	pub fn exists(&self, path: &Path) -> bool {
		path.exists()
	}

	pub fn is_dir(&self, path: &Path) -> bool {
		self.watch_map.get(path).cloned().unwrap_or_else(|| path.is_dir())
	}

	pub fn is_file(&self, path: &Path) -> bool {
		!self.is_dir(path)
	}

	pub fn receiver(&self) -> Receiver<VfsEvent> {
		self.watcher.receiver()
	}

	pub fn process_event(&mut self, event: &VfsEvent) {
		if let VfsEvent::Delete(path) = event {
			self.unwatch(path).ok();
		}
	}
}
