use anyhow::Result;
use crossbeam_channel::Receiver;
use notify::{RecursiveMode::NonRecursive, Watcher};
use std::path::Path;

use super::{debouncer::VfsDebouncer, VfsEvent};

pub struct VfsWatcher {
	debouncer: VfsDebouncer,
	receiver: Receiver<VfsEvent>,
}

impl VfsWatcher {
	pub fn new() -> Result<Self> {
		let (sender, receiver) = crossbeam_channel::unbounded();
		let debouncer = VfsDebouncer::new(sender)?;

		Ok(Self { debouncer, receiver })
	}

	pub fn watch(&mut self, path: &Path) -> Result<()> {
		self.debouncer.inner.watcher().watch(path, NonRecursive)?;
		self.debouncer.inner.cache().add_root(path, NonRecursive);

		Ok(())
	}

	pub fn unwatch(&mut self, path: &Path) -> Result<()> {
		self.debouncer.inner.watcher().unwatch(path)?;
		self.debouncer.inner.cache().remove_root(path);

		Ok(())
	}

	pub fn receiver(&self) -> Receiver<VfsEvent> {
		self.receiver.clone()
	}
}
