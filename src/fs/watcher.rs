#![allow(clippy::type_complexity)]

use super::debouncer::FsDebouncer;
use anyhow::Result;
use crossbeam_channel::Sender;
use notify::{Error, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, FileIdMap};
use std::{
	path::PathBuf,
	sync::{
		mpsc::{self, Receiver},
		Arc, Mutex,
	},
	thread::{self},
	time::Duration,
	vec,
};

pub struct FsWatcher {
	debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
	fs_debouncer: Arc<Mutex<FsDebouncer>>,
	receiver: Arc<Mutex<Receiver<Result<Vec<DebouncedEvent>, Vec<Error>>>>>,
	watched_paths: Vec<PathBuf>,
}

impl FsWatcher {
	pub fn new(root: &PathBuf, handler: &Sender<WorkspaceEvent>) -> Result<Self> {
		let (sender, receiver) = mpsc::channel();
		let mut debouncer = new_debouncer(Duration::from_millis(100), None, sender, false)?;
		let fs_debouncer = FsDebouncer::new(root, handler);

		debouncer.watcher().watch(root, RecursiveMode::NonRecursive)?;
		debouncer.cache().add_root(root, RecursiveMode::NonRecursive);

		let receiver = Arc::new(Mutex::new(receiver));
		let fs_debouncer = Arc::new(Mutex::new(fs_debouncer));

		Ok(Self {
			debouncer,
			fs_debouncer,
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

			self.watched_paths.retain(|p| p != path);
		}

		Ok(())
	}

	pub fn start(&self) -> Result<()> {
		let receiver = self.receiver.clone();
		let fs_debouncer = self.fs_debouncer.clone();

		thread::spawn(move || {
			let receiver = receiver.lock().unwrap();

			#[cfg(not(target_os = "linux"))]
			let fs_debouncer = fs_debouncer.lock().unwrap();

			#[cfg(target_os = "linux")]
			let mut fs_debouncer = fs_debouncer.lock().unwrap();

			for response in receiver.iter() {
				for event in response.unwrap() {
					fs_debouncer.debounce(&event);
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
