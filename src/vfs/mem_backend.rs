use crossbeam_channel::Receiver;
use std::{
	collections::HashMap,
	io::{Error, ErrorKind, Result},
	path::{Path, PathBuf},
};

use super::{VfsBackend, VfsEvent};

#[derive(Debug)]
pub enum VfsEntry {
	Directory(Vec<PathBuf>),
	File(Vec<u8>),
}

pub struct MemBackend {
	inner: HashMap<PathBuf, VfsEntry>,
	receiver: Receiver<VfsEvent>,
}

impl MemBackend {
	pub fn new() -> Self {
		let (_sender, receiver) = crossbeam_channel::unbounded();

		Self {
			inner: HashMap::new(),
			receiver,
		}
	}

	pub fn get_entry(&self, path: &Path) -> Result<&VfsEntry> {
		match self.inner.get(path) {
			Some(entry) => Ok(entry),
			None => not_found(path),
		}
	}
}

impl VfsBackend for MemBackend {
	fn read(&self, path: &Path) -> Result<Vec<u8>> {
		match self.get_entry(path)? {
			VfsEntry::File(contents) => Ok(contents.clone()),
			VfsEntry::Directory(_) => not_file(path),
		}
	}

	fn read_to_string(&self, path: &Path) -> Result<String> {
		match self.get_entry(path)? {
			VfsEntry::File(contents) => {
				Ok(String::from_utf8(contents.clone()).map_err(|err| Error::new(ErrorKind::InvalidData, err))?)
			}
			VfsEntry::Directory(_) => not_file(path),
		}
	}

	fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
		match self.get_entry(path)? {
			VfsEntry::File(_) => not_dir(path),
			VfsEntry::Directory(children) => Ok(children.clone()),
		}
	}

	fn write(&mut self, path: &Path, contents: &[u8]) -> Result<()> {
		let entry = self.inner.entry(path.to_owned()).or_insert(VfsEntry::File(Vec::new()));

		match entry {
			VfsEntry::File(old) => contents.clone_into(old),
			VfsEntry::Directory(_) => return not_file(path),
		}

		Ok(())
	}

	fn create_dir(&mut self, path: &Path) -> Result<()> {
		let mut cur_path = PathBuf::new();
		let mut last_path = PathBuf::new();

		for comp in path.components() {
			cur_path.push(comp);

			match self.inner.get(&cur_path) {
				Some(VfsEntry::File(_)) => return not_dir(&cur_path),
				Some(VfsEntry::Directory(_)) => (),
				None => {
					self.inner.insert(cur_path.clone(), VfsEntry::Directory(Vec::new()));

					if let Some(VfsEntry::Directory(children)) = self.inner.get_mut(&last_path) {
						children.push(cur_path.clone());
					}
				}
			}

			last_path.push(comp);
		}

		Ok(())
	}

	fn rename(&mut self, from: &Path, to: &Path) -> Result<()> {
		let entry = self.inner.remove(from);

		match entry {
			Some(entry) => {
				self.inner.insert(to.to_owned(), entry);

				if let Some(VfsEntry::Directory(children)) = self.inner.get_mut(from.parent().unwrap()) {
					children.retain(|p| p != from);
					children.push(to.to_owned());
				}
			}
			None => return not_found(from),
		}

		Ok(())
	}

	fn remove(&mut self, path: &Path) -> Result<()> {
		let entry = self.inner.remove(path);

		match entry {
			Some(VfsEntry::Directory(_)) => {
				self.inner.retain(|p, _| !p.starts_with(path));
			}
			None => return not_found(path),
			_ => {}
		}

		Ok(())
	}

	fn exists(&self, path: &Path) -> bool {
		self.inner.contains_key(path)
	}

	fn is_dir(&self, path: &Path) -> bool {
		matches!(self.inner.get(path), Some(VfsEntry::Directory(_)))
	}

	fn is_file(&self, path: &Path) -> bool {
		matches!(self.inner.get(path), Some(VfsEntry::File(_)))
	}

	fn watch(&mut self, _path: &Path, _recursive: bool) -> Result<()> {
		Ok(())
	}

	fn unwatch(&mut self, _path: &Path) -> Result<()> {
		Ok(())
	}

	fn pause(&mut self) {}

	fn resume(&mut self) {}

	fn receiver(&self) -> Receiver<VfsEvent> {
		self.receiver.clone()
	}
}

// Based on Rojo's in_memory_fs::not_found (https://github.com/rojo-rbx/rojo/blob/master/crates/memofs/src/in_memory_fs.rs)
fn not_found<T>(path: &Path) -> Result<T> {
	Err(Error::new(
		ErrorKind::NotFound,
		format!("path {} not found", path.display()),
	))
}

fn not_file<T>(path: &Path) -> Result<T> {
	Err(Error::other(format!(
		"path {} was a directory, but must be a file",
		path.display()
	)))
}

fn not_dir<T>(path: &Path) -> Result<T> {
	Err(Error::other(format!(
		"path {} was a file, but must be a directory",
		path.display()
	)))
}
