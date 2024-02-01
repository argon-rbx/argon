use anyhow::Result;
use crossbeam_channel::Receiver;
use std::{
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
	sync::Mutex,
};

use self::backend::VfsBackend;
use crate::lock;

pub mod backend;
pub mod debouncer;
pub mod watcher;

#[derive(Debug, Clone)]
pub enum VfsEvent {
	Create(PathBuf),
	Delete(PathBuf),
	Write(PathBuf),
}

pub struct Vfs {
	inner: Mutex<VfsBackend>,
}

impl Vfs {
	pub fn new() -> Self {
		Self {
			inner: Mutex::new(VfsBackend::new()),
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

	pub fn reader(&self, path: &Path) -> Result<BufReader<File>> {
		lock!(self.inner).reader(path)
	}

	pub fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
		lock!(self.inner).read_dir(path)
	}

	pub fn exists(&self, path: &Path) -> bool {
		lock!(self.inner).exists(path)
	}

	pub fn is_watched(&self, path: &Path) -> bool {
		lock!(self.inner).is_watched(path)
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
