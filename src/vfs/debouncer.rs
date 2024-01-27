use crossbeam_channel::Sender;
use notify::{
	event::{DataChange, ModifyKind},
	EventKind, RecommendedWatcher,
};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, FileIdMap};
use std::{sync::mpsc, thread, time::Duration};

use super::VfsEvent;

macro_rules! event_path {
	($event:expr) => {
		$event.paths.first().unwrap().to_owned()
	};
}

pub struct VfsDebouncer {
	pub inner: Debouncer<RecommendedWatcher, FileIdMap>,
}

impl VfsDebouncer {
	pub fn new(handler: Sender<VfsEvent>) -> Self {
		let (sender, receiver) = mpsc::channel();
		let debouncer = new_debouncer(Duration::from_millis(100), None, sender, false).unwrap();

		thread::spawn(move || {
			for events in receiver {
				for event in events.unwrap() {
					if let Some(event) = Self::debounce(&event) {
						handler.send(event).unwrap();
					}
				}
			}
		});

		Self { inner: debouncer }
	}

	#[cfg(target_os = "macos")]
	fn debounce(event: &DebouncedEvent) -> Option<VfsEvent> {
		match event.kind {
			EventKind::Create(_) => {
				let path = event_path!(event);

				if path.exists() {
					Some(VfsEvent::Create(path))
				} else {
					None
				}
			}
			EventKind::Modify(kind) => match kind {
				ModifyKind::Name(_) => {
					let path = event_path!(event);

					if path.exists() {
						Some(VfsEvent::Create(path))
					} else {
						Some(VfsEvent::Delete(path))
					}
				}
				ModifyKind::Data(kind) => {
					if kind == DataChange::Content {
						Some(VfsEvent::Write(event_path!(event)))
					} else {
						None
					}
				}
				_ => None,
			},
			_ => None,
		}
	}

	// TODO
	#[cfg(target_os = "linux")]
	fn debounce(&mut self, event: &DebouncedEvent) {
		match event.kind {
			EventKind::Create(_) => {
				let path = self.get_path(event);

				self.time = event.time;
				self.path = path.to_owned();

				self.send(FsEventKind::Create, path);
			}
			EventKind::Modify(ModifyKind::Name(mode)) => match mode {
				RenameMode::From => {
					self.send(FsEventKind::Delete, self.get_path(event));
				}
				RenameMode::To => {
					self.send(FsEventKind::Create, self.get_path(event));
				}
				_ => {}
			},
			EventKind::Access(kind) => {
				if kind == AccessKind::Close(AccessMode::Write) {
					let duration = event.time.duration_since(self.time);
					let path = self.get_path(event);

					if duration < DEBOUNCE_TIME && path == self.path {
						return;
					}

					self.send(FsEventKind::Write, path);
				}
			}
			_ => {}
		}
	}

	#[cfg(target_os = "windows")]
	fn debounce(event: &DebouncedEvent) -> Option<VfsEvent> {
		match event.kind {
			EventKind::Create(_) => Some(VfsEvent::Create(event_path!(event))),
			EventKind::Remove(_) => Some(VfsEvent::Delete(event_path!(event))),
			EventKind::Modify(_) => Some(VfsEvent::Write(event_path!(event))),
			_ => None,
		}
	}
}
