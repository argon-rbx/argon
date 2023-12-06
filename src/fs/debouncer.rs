use super::watcher::{WorkspaceEvent, WorkspaceEventKind};
use crossbeam_channel::Sender;
use notify::EventKind;
use notify_debouncer_full::DebouncedEvent;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use notify::event::{DataChange, ModifyKind};

#[cfg(target_os = "linux")]
use {
	notify::event::{AccessKind, AccessMode, ModifyKind, RenameMode},
	std::time::{Duration, Instant},
};

#[cfg(target_os = "linux")]
const DEBOUNCE_TIME: Duration = Duration::from_micros(500);

pub struct FsDebouncer {
	root: PathBuf,
	sender: Sender<WorkspaceEvent>,

	#[cfg(target_os = "linux")]
	time: Instant,
	#[cfg(target_os = "linux")]
	path: PathBuf,
}

impl FsDebouncer {
	pub fn new(root: &PathBuf, sender: &Sender<WorkspaceEvent>) -> Self {
		Self {
			root: root.to_owned(),
			sender: sender.to_owned(),

			#[cfg(target_os = "linux")]
			time: Instant::now(),
			#[cfg(target_os = "linux")]
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
					if kind == DataChange::Content {
						self.send(WorkspaceEventKind::Write, self.get_path(event));
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
		match event.kind {
			EventKind::Create(_) => {
				self.send(WorkspaceEventKind::Create, self.get_path(event));
			}
			EventKind::Remove(_) => {
				self.send(WorkspaceEventKind::Delete, self.get_path(event));
			}
			EventKind::Modify(_) => {
				self.send(WorkspaceEventKind::Write, self.get_path(event));
			}
			_ => {}
		}
	}
}
