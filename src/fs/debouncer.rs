use super::watcher::{WorkspaceEvent, WorkspaceEventKind};
use notify::{
	event::{DataChange, ModifyKind},
	EventKind,
};
use notify_debouncer_full::DebouncedEvent;
use std::{path::PathBuf, sync::mpsc::Sender};

pub struct ArgonDebouncer {
	root: PathBuf,
	sender: Sender<WorkspaceEvent>,
}

impl ArgonDebouncer {
	pub fn new(root: &PathBuf, sender: &Sender<WorkspaceEvent>) -> Self {
		Self {
			root: root.to_owned(),
			sender: sender.to_owned(),
		}
	}

	#[cfg(target_os = "macos")]
	pub fn debounce(&self, event: &DebouncedEvent) {
		let get_path = || event.paths.first().unwrap().to_owned();

		let send = |kind: WorkspaceEventKind, path: PathBuf| {
			let parent = path.parent().unwrap();
			let root = self.root == parent;

			let event = WorkspaceEvent { kind, path, root };

			self.sender.send(event).unwrap();
		};

		match event.kind {
			EventKind::Create(_) => {
				send(WorkspaceEventKind::Create, get_path());
			}
			EventKind::Modify(kind) => match kind {
				ModifyKind::Name(_) => {
					let path = get_path();

					if path.exists() {
						send(WorkspaceEventKind::Create, path);
					} else {
						send(WorkspaceEventKind::Delete, path);
					}
				}
				ModifyKind::Data(kind) => {
					let path = get_path();

					if kind == DataChange::Content && path.is_file() {
						send(WorkspaceEventKind::Write, path);
					}
				}
				_ => {}
			},
			_ => {}
		}
	}

	#[cfg(target_os = "linux")]
	pub fn debounce(&self, event: &DebouncedEvent) {
		println!("{:?}", event);
	}

	#[cfg(target_os = "windows")]
	pub fn debounce() {
		unimplemented!()
	}
}
