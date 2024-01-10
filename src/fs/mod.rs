mod debouncer;
mod watcher;

use anyhow::Result;
use crossbeam_channel::Receiver;
use std::path::PathBuf;

use self::watcher::FsWatcher;

#[derive(Debug)]
pub struct FsEvent {
	pub kind: FsEventKind,
	pub path: PathBuf,
	pub root: bool,
}

#[derive(Debug)]
pub enum FsEventKind {
	Create,
	Delete,
	Write,
}

pub struct Fs {
	watcher: FsWatcher,
	receiver: Receiver<FsEvent>,
}

impl Fs {
	pub fn new(root: &PathBuf) -> Result<Self> {
		let (sender, receiver) = crossbeam_channel::unbounded();
		let watcher = FsWatcher::new(root, &sender)?;

		watcher.start()?;

		Ok(Self { watcher, receiver })
	}

	pub fn watch(&mut self, path: &PathBuf) -> Result<()> {
		self.watcher.watch(path)
	}

	pub fn unwatch(&mut self, path: &PathBuf) -> Result<()> {
		self.watcher.unwatch(path)
	}

	pub fn watch_all(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
		for path in paths {
			self.watch(path)?;
		}

		Ok(())
	}

	pub fn unwatch_all(&mut self, paths: &Vec<PathBuf>) -> Result<()> {
		for path in paths {
			self.unwatch(path)?;
		}

		Ok(())
	}

	pub fn receiver(&self) -> Receiver<FsEvent> {
		self.receiver.clone()
	}
}
