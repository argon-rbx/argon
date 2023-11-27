use anyhow::Result;
use notify::{Error, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
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

pub struct WorkspaceWatcher {
	root: PathBuf,
	debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
	receiver: WorkspaceReceiver,
	watched_paths: Vec<PathBuf>,
}

type WorkspaceReceiver = Arc<Mutex<Receiver<Result<Vec<DebouncedEvent>, Vec<Error>>>>>;

impl WorkspaceWatcher {
	pub fn new(root: PathBuf) -> Result<Self> {
		let (sender, receiver) = mpsc::channel();
		let mut debouncer = new_debouncer(Duration::from_millis(100), None, sender)?;

		debouncer.watcher().watch(&root, RecursiveMode::NonRecursive)?;
		debouncer.cache().add_root(&root, RecursiveMode::NonRecursive);

		let receiver = Arc::new(Mutex::new(receiver));

		Ok(Self {
			root,
			debouncer,
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

	pub fn start(&self, sender: Sender<WorkspaceEvent>) -> Result<()> {
		let receiver = self.receiver.clone();

		thread::spawn(move || {
			let receiver = receiver.lock().unwrap();

			for response in receiver.iter() {
				for event in response.unwrap() {
					// TEMP!
					sender
						.send(WorkspaceEvent {
							kind: WorkspaceEventKind::Create,
							paths: vec![],
							root: false,
						})
						.unwrap();

					println!("{:?}, {:?}", event.kind, event.paths);
				}
			}
		});

		Ok(())
	}
}

#[derive(Debug)]
pub struct WorkspaceEvent {
	pub kind: WorkspaceEventKind,
	pub paths: Vec<PathBuf>,
	pub root: bool,
}

#[derive(Debug)]
pub enum WorkspaceEventKind {
	Create,
	Rename,
	Delete,
	Write,
}
