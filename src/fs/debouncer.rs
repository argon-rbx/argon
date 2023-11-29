use super::watcher::{WorkspaceEvent, WorkspaceEventKind};
use notify::{event::ModifyKind, EventKind};
use notify_debouncer_full::DebouncedEvent;
use std::{
	path::PathBuf,
	sync::mpsc::Sender,
	time::{Duration, Instant},
};

#[cfg(target_os = "macos")]
use notify::event::DataChange;

#[cfg(target_os = "linux")]
use notify::event::{AccessKind, AccessMode, RenameMode};

#[allow(dead_code)]
const DEBOUNCE_TIME: Duration = Duration::from_micros(500);

pub struct ArgonDebouncer {
	root: PathBuf,
	sender: Sender<WorkspaceEvent>,

	#[allow(dead_code)]
	time: Instant,

	#[allow(dead_code)]
	path: PathBuf,
}

impl ArgonDebouncer {
	pub fn new(root: &PathBuf, sender: &Sender<WorkspaceEvent>) -> Self {
		Self {
			root: root.to_owned(),
			sender: sender.to_owned(),
			time: Instant::now(),
			path: PathBuf::new(),
		}
	}

	fn get_path(&self, event: &DebouncedEvent) -> PathBuf {
		event.paths.first().unwrap().to_owned()
	}

	fn send(&self, kind: WorkspaceEventKind, path: PathBuf) {
		let parent = path.parent().unwrap();
		let root = self.root == parent;

		let event = WorkspaceEvent { kind, path, root };

		self.sender.send(event).unwrap();
	}

	#[cfg(target_os = "macos")]
	pub fn debounce(&self, event: &DebouncedEvent) {
		match event.kind {
			EventKind::Create(_) => {
				let path = self.get_path(event);

				if path.exists() {
					self.send(WorkspaceEventKind::Create, path);
				}
			}
			EventKind::Modify(kind) => match kind {
				ModifyKind::Name(_) => {
					let path = self.get_path(event);

					if path.exists() {
						self.send(WorkspaceEventKind::Create, path);
					} else {
						self.send(WorkspaceEventKind::Delete, path);
					}
				}
				ModifyKind::Data(kind) => {
					let path = self.get_path(event);

					if kind == DataChange::Content && path.is_file() {
						self.send(WorkspaceEventKind::Write, path);
					}
				}
				_ => {}
			},
			_ => {}
		}
	}

	#[cfg(target_os = "linux")]
	pub fn debounce(&mut self, event: &DebouncedEvent) {
		match event.kind {
			EventKind::Create(_) => {
				let path = self.get_path(event);

				self.time = event.time;
				self.path = path.to_owned();

				self.send(WorkspaceEventKind::Create, path);
			}
			EventKind::Modify(ModifyKind::Name(mode)) => match mode {
				RenameMode::From => {
					self.send(WorkspaceEventKind::Delete, self.get_path(event));
				}
				RenameMode::To => {
					self.send(WorkspaceEventKind::Create, self.get_path(event));
				}
				_ => {}
			},
			EventKind::Access(kind) => {
				if kind == AccessKind::Close(AccessMode::Write) {
					let duration = event.time.duration_since(self.time);
					let path = self.get_path(event);

					println!("{:?}", duration);

					if duration < DEBOUNCE_TIME && path == self.path {
						return;
					}

					self.send(WorkspaceEventKind::Write, path);
				}
			}
			_ => {}
		}
	}

	#[cfg(target_os = "windows")]
	pub fn debounce(&self, event: &DebouncedEvent) {
		println!("{:?}", event);
	}
}
