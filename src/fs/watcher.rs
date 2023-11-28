#![allow(clippy::type_complexity)]

use super::debouncer::ArgonDebouncer;
use anyhow::Result;
use notify::{Error, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, FileIdMap};
use std::{
	path::PathBuf,
	sync::{
		mpsc::{self, Receiver, Sender},
		Arc, Mutex,
	},
	thread::{self},
	time::Duration,
	vec,
};

pub struct ArgonWatcher {
	debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
	argon_debouncer: Arc<ArgonDebouncer>,
	receiver: Arc<Mutex<Receiver<Result<Vec<DebouncedEvent>, Vec<Error>>>>>,
	watched_paths: Vec<PathBuf>,
}

impl ArgonWatcher {
	pub fn new(root: &PathBuf, handler: &Sender<WorkspaceEvent>) -> Result<Self> {
		let (sender, receiver) = mpsc::channel();
		let mut debouncer = new_debouncer(Duration::from_millis(100), None, sender, false)?;
		let argon_debouncer = Arc::new(ArgonDebouncer::new(root, handler));

		debouncer.watcher().watch(root, RecursiveMode::NonRecursive)?;
		debouncer.cache().add_root(root, RecursiveMode::NonRecursive);

		let receiver = Arc::new(Mutex::new(receiver));

		Ok(Self {
			debouncer,
			argon_debouncer,
			receiver,
			watched_paths: vec![],
		})
	}

	pub fn watch(&mut self, path: &PathBuf) -> Result<()> {
		if !self.watched_paths.contains(path) && path.exists() {
			self.debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
			self.debouncer.cache().add_root(path, RecursiveMode::Recursive);

			self.watched_paths.push(path.to_owned());
		}

		Ok(())
	}

	pub fn unwatch(&mut self, path: &PathBuf) -> Result<()> {
		if self.watched_paths.contains(path) {
			self.debouncer.watcher().unwatch(path)?;
			self.debouncer.cache().remove_root(path);

			let index = self.watched_paths.iter().position(|p| p == path).unwrap();
			self.watched_paths.remove(index);
		}

		Ok(())
	}

	pub fn start(&self) -> Result<()> {
		let receiver = self.receiver.clone();
		let argon_debouncer = self.argon_debouncer.clone();

		thread::spawn(move || {
			let receiver = receiver.lock().unwrap();

			for response in receiver.iter() {
				for event in response.unwrap() {
					argon_debouncer.debounce(&event);
				}
			}
		});

		Ok(())
	}
}

#[derive(Debug)]
pub struct WorkspaceEvent {
	pub kind: WorkspaceEventKind,
	pub path: PathBuf,
	pub root: bool,
}

#[derive(Debug)]
pub enum WorkspaceEventKind {
	Create,
	Delete,
	Write,
}
