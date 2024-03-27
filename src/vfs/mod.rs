use crossbeam_channel::Receiver;
use std::{
	io::Result,
	path::{Path, PathBuf},
	sync::Mutex,
};

use self::{mem_backend::MemBackend, std_backend::StdBackend};
use crate::lock;

pub mod debouncer;
pub mod mem_backend;
pub mod std_backend;

#[derive(Debug, Clone)]
pub enum VfsEvent {
	Create(PathBuf),
	Delete(PathBuf),
	Write(PathBuf),
}

pub trait VfsBackend: Send {
	fn read(&self, path: &Path) -> Result<Vec<u8>>;
	fn read_to_string(&self, path: &Path) -> Result<String>;
	fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

	fn write(&mut self, path: &Path, contents: &[u8]) -> Result<()>;
	fn create_dir(&mut self, path: &Path) -> Result<()>;
	fn remove(&mut self, path: &Path) -> Result<()>;

	fn exists(&self, path: &Path) -> bool;
	fn is_dir(&self, path: &Path) -> bool;
	fn is_file(&self, path: &Path) -> bool;

	fn watch(&mut self, path: &Path) -> Result<()>;
	fn unwatch(&mut self, path: &Path) -> Result<()>;

	fn receiver(&self) -> Receiver<VfsEvent>;
}

impl VfsEvent {
	pub fn path(&self) -> &Path {
		match self {
			VfsEvent::Create(path) | VfsEvent::Delete(path) | VfsEvent::Write(path) => path.as_ref(),
		}
	}
}

pub struct Vfs {
	inner: Mutex<Box<dyn VfsBackend>>,
}

impl Vfs {
	pub fn new(watch: bool) -> Self {
		Self {
			inner: Mutex::new(Box::new(StdBackend::new(watch))),
		}
	}

	pub fn new_virtual() -> Self {
		Self {
			inner: Mutex::new(Box::new(MemBackend::new())),
		}
	}

	pub fn read(&self, path: &Path) -> Result<Vec<u8>> {
		lock!(self.inner).read(path)
	}

	pub fn read_to_string(&self, path: &Path) -> Result<String> {
		lock!(self.inner).read_to_string(path)
	}

	pub fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
		lock!(self.inner).read_dir(path)
	}

	pub fn write(&self, path: &Path, contents: &[u8]) -> Result<()> {
		lock!(self.inner).write(path, contents)
	}

	pub fn create_dir(&self, path: &Path) -> Result<()> {
		lock!(self.inner).create_dir(path)
	}

	pub fn remove(&self, path: &Path) -> Result<()> {
		lock!(self.inner).remove(path)
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

	pub fn watch(&self, path: &Path) -> Result<()> {
		lock!(self.inner).watch(path)
	}

	pub fn unwatch(&self, path: &Path) -> Result<()> {
		lock!(self.inner).unwatch(path)
	}

	pub fn receiver(&self) -> Receiver<VfsEvent> {
		lock!(self.inner).receiver()
	}
}
