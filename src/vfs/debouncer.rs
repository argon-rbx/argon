use crossbeam_channel::Receiver;
use log::trace;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, FileIdMap};
use std::{
	io::{self, Result},
	path::Path,
	sync::{mpsc, Arc, RwLock},
	thread::Builder,
	time::{Duration, Instant},
};

#[cfg(target_os = "macos")]
use notify::event::DataChange;

#[cfg(not(target_os = "windows"))]
use notify::event::ModifyKind;

#[cfg(target_os = "linux")]
use {
	notify::event::{AccessKind, AccessMode, RenameMode},
	std::path::PathBuf,
};

use super::VfsEvent;
use crate::constants::SYNCBACK_DEBOUNCE_TIME;

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
	inner: Debouncer<RecommendedWatcher, FileIdMap>,
	pause_state: Arc<RwLock<(bool, Instant)>>,
	receiver: Receiver<VfsEvent>,
}

impl VfsDebouncer {
	pub fn new() -> Self {
		let (inner_sender, inner_receiver) = mpsc::channel();
		let (sender, receiver) = crossbeam_channel::unbounded();

		let debouncer = new_debouncer(Duration::from_millis(100), None, inner_sender, false).unwrap();

		let pause_state = Arc::new(RwLock::new((false, Instant::now())));
		let local_pause_state = pause_state.clone();

		Builder::new()
			.name("debouncer".to_owned())
			.spawn(move || {
				#[cfg(target_os = "linux")]
				let mut context = DebounceContext {
					time: Instant::now(),
					path: PathBuf::new(),
				};

				for events in inner_receiver {
					let (is_paused, timestamp) = *local_pause_state.read().unwrap();

					if is_paused || timestamp.elapsed() < SYNCBACK_DEBOUNCE_TIME {
						continue;
					}

					for event in events.unwrap() {
						trace!("Debouncing event, paths: {:?}, kind: {:?}", event.paths, event.kind);

						#[cfg(not(target_os = "linux"))]
						if let Some(event) = debounce(&event) {
							sender.send(event).unwrap();
						}

						#[cfg(target_os = "linux")]
						if let Some(event) = debounce(&event, &mut context) {
							sender.send(event).unwrap();
						}
					}
				}
			})
			.unwrap();

		Self {
			inner: debouncer,
			pause_state,
			receiver,
		}
	}

	pub fn watch(&mut self, path: &Path, recursive: bool) -> Result<()> {
		let recursive = if recursive {
			RecursiveMode::Recursive
		} else {
			RecursiveMode::NonRecursive
		};

		self.inner.watcher().watch(path, recursive).map_err(map_error)?;
		self.inner.cache().add_root(path, recursive);

		Ok(())
	}

	pub fn unwatch(&mut self, path: &Path) -> Result<()> {
		self.inner.watcher().unwatch(path).map_err(map_error)?;
		self.inner.cache().remove_root(path);

		Ok(())
	}

	pub fn pause(&mut self) {
		*self.pause_state.write().unwrap() = (true, Instant::now());
	}

	pub fn resume(&mut self) {
		*self.pause_state.write().unwrap() = (false, Instant::now());
	}

	pub fn receiver(&self) -> Receiver<VfsEvent> {
		self.receiver.clone()
	}
}

fn map_error(err: notify::Error) -> io::Error {
	match err.kind {
		notify::ErrorKind::Io(err) => err,
		notify::ErrorKind::PathNotFound => io::Error::new(io::ErrorKind::NotFound, err),
		notify::ErrorKind::WatchNotFound => io::Error::new(io::ErrorKind::NotFound, err),
		_ => io::Error::other(err),
	}
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
			context.path.clone_from(&path);

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
