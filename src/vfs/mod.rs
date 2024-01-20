use anyhow::Result;
use crossbeam_channel::Receiver;
use std::path::{Path, PathBuf};

use self::watcher::VfsWatcher;

pub mod debouncer;
pub mod watcher;

pub enum VfsEvent {
	Create(PathBuf),
	Delete(PathBuf),
	Write(PathBuf),
}

pub struct Vfs {
	watcher: VfsWatcher,
	watched_paths: Vec<PathBuf>,
}

impl Vfs {
	pub fn new(auto_watch: bool) -> Result<Self> {
		Ok(Self {
			watcher: VfsWatcher::new()?,
			watched_paths: Vec::new(),
		})
	}

	pub fn watch(&mut self, path: &Path) -> Result<()> {
		self.watcher.watch(path)?;
		self.watched_paths.push(path.to_owned());

		if path.is_dir() {
			for entry in path.read_dir()? {
				self.watch(&entry?.path())?;
			}
		}

		Ok(())
	}

	pub fn unwatch(&mut self, path: &Path) -> Result<()> {
		self.watcher.unwatch(path)?;

		let mut unwatched = vec![];

		for (index, path) in self.watched_paths.iter().enumerate() {
			if path.starts_with(path) {
				self.watcher.unwatch(path)?;

				unwatched.push(index);
			}
		}

		for index in unwatched {
			self.watched_paths.remove(index);
		}

		Ok(())
	}

	pub fn receiver(&self) -> Receiver<VfsEvent> {
		self.watcher.receiver()
	}
}
