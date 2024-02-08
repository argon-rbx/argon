use crossbeam_channel::Sender;
use notify::{EventKind, RecommendedWatcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, FileIdMap};
use std::{sync::mpsc, thread::Builder, time::Duration};

#[cfg(target_os = "macos")]
use notify::event::DataChange;

#[cfg(not(target_os = "windows"))]
use notify::event::ModifyKind;

#[cfg(target_os = "linux")]
use {
	notify::event::{AccessKind, AccessMode, RenameMode},
	std::{path::PathBuf, time::Instant},
};

use super::VfsEvent;

#[cfg(target_os = "linux")]
const DEBOUNCE_TIME: Duration = Duration::from_micros(500);

macro_rules! event_path {
	($event:expr) => {
		$event.paths.first().unwrap().to_owned()
	};
}

#[cfg(target_os = "linux")]
struct DebounceContext {
	time: Instant,
	path: PathBuf,
}

pub struct VfsDebouncer {
	pub inner: Debouncer<RecommendedWatcher, FileIdMap>,
}

impl VfsDebouncer {
	pub fn new(handler: Sender<VfsEvent>) -> Self {
		let (sender, receiver) = mpsc::channel();
		let debouncer = new_debouncer(Duration::from_millis(100), None, sender, false).unwrap();

		Builder::new()
			.name("debouncer".to_owned())
			.spawn(move || {
				#[cfg(target_os = "linux")]
				let mut context = DebounceContext {
					time: Instant::now(),
					path: PathBuf::new(),
				};

				for events in receiver {
					for event in events.unwrap() {
						#[cfg(not(target_os = "linux"))]
						if let Some(event) = Self::debounce(&event) {
							handler.send(event).unwrap();
						}

						#[cfg(target_os = "linux")]
						if let Some(event) = Self::debounce(&event, &mut context) {
							handler.send(event).unwrap();
						}
					}
				}
			})
			.unwrap();

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

	#[cfg(target_os = "linux")]
	fn debounce(event: &DebouncedEvent, context: &mut DebounceContext) -> Option<VfsEvent> {
		match event.kind {
			EventKind::Create(_) => {
				let path = event_path!(event);

				context.time = event.time;
				context.path = path.clone();

				Some(VfsEvent::Create(path))
			}
			EventKind::Modify(ModifyKind::Name(mode)) => match mode {
				RenameMode::From => Some(VfsEvent::Delete(event_path!(event))),
				RenameMode::To => Some(VfsEvent::Create(event_path!(event))),
				_ => None,
			},
			EventKind::Access(kind) => {
				if kind == AccessKind::Close(AccessMode::Write) {
					let duration = event.time.duration_since(context.time);
					let path = event_path!(event);

					if duration < DEBOUNCE_TIME && path == context.path {
						return None;
					}

					Some(VfsEvent::Write(event_path!(event)))
				} else {
					None
				}
			}
			_ => None,
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
