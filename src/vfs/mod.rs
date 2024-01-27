use anyhow::Result;
use crossbeam_channel::Receiver;
use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	sync::Mutex,
};

use crate::lock;

use self::watcher::VfsWatcher;

pub mod debouncer;
pub mod watcher;

#[derive(Debug, Clone)]
pub enum VfsEvent {
	Create(PathBuf),
	Delete(PathBuf),
	Write(PathBuf),
}

struct VfsInner {
	watcher: VfsWatcher,
	// bool - is_dir?
	watch_map: HashMap<PathBuf, bool>,
}

impl VfsInner {
	pub fn new() -> Self {
		Self {
			watcher: VfsWatcher::new(),
			watch_map: HashMap::new(),
		}
	}

	pub fn watch(&mut self, path: &Path) -> Result<()> {
		if self.watch_map.contains_key(path) {
			return Ok(());
		}

		self.watcher.watch(path)?;
		self.watch_map.insert(path.to_owned(), path.is_dir());

		Ok(())
	}

	pub fn unwatch(&mut self, path: &Path) -> Result<()> {
		if !self.watch_map.contains_key(path) {
			return Ok(());
		}

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

	pub fn read(&self, path: &Path) -> Result<String> {
		let file = fs::read_to_string(path)?;
		Ok(file)
	}

	pub fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
		let mut paths = vec![];

		for entry in fs::read_dir(path)? {
			paths.push(entry?.path());
		}

		Ok(paths)
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

pub struct Vfs {
	inner: Mutex<VfsInner>,
}

impl Vfs {
	pub fn new() -> Self {
		Self {
			inner: Mutex::new(VfsInner::new()),
		}
	}

	pub fn watch(&self, path: &Path) -> Result<()> {
		lock!(self.inner).watch(path)
	}

	pub fn unwatch(&self, path: &Path) -> Result<()> {
		lock!(self.inner).unwatch(path)
	}

	pub fn read(&self, path: &Path) -> Result<String> {
		lock!(self.inner).read(path)
	}

	pub fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
		lock!(self.inner).read_dir(path)
	}

	pub fn exists(&self, path: &Path) -> bool {
		lock!(self.inner).exists(path)
	}

	pub fn is_dir(&self, path: &Path) -> bool {
		lock!(self.inner).is_dir(path)
	}

	pub fn is_file(&self, path: &Path) -> bool {
		lock!(self.inner).is_file(path)
	}

	pub fn receiver(&self) -> Receiver<VfsEvent> {
		lock!(self.inner).receiver()
	}

	pub fn process_event(&self, event: &VfsEvent) {
		lock!(self.inner).process_event(event)
	}
}
