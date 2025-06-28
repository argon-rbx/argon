use anyhow::{bail, Result};
use colored::Colorize;
use crossbeam_channel::{Receiver, Sender};
use std::{collections::HashMap, sync::RwLock};

use crate::{
	argon_warn,
	config::Config,
	constants::QUEUE_TIMEOUT,
	server::{self, Message},
};

macro_rules! read {
	($rwlock:expr) => {
		$rwlock.write().unwrap()
	};
}

macro_rules! write {
	($rwlock:expr) => {
		$rwlock.write().unwrap()
	};
}

#[derive(Debug, Clone)]
struct Listener {
	pub id: u32,
	pub name: String,
	pub is_internal: bool,
}

#[derive(Debug)]
struct Channel {
	sender: Sender<Message>,
	receiver: Receiver<Message>,
}

#[derive(Debug)]
pub struct Queue {
	queues: RwLock<HashMap<u32, Channel>>,
	listeners: RwLock<Vec<Listener>>,
	unsynced_changes: RwLock<usize>,
}

impl Queue {
	pub fn new() -> Self {
		Self {
			queues: RwLock::new(HashMap::new()),
			listeners: RwLock::new(Vec::new()),
			unsynced_changes: RwLock::new(0),
		}
	}

	pub fn push<M>(&self, message: M, id: Option<u32>) -> Result<()>
	where
		M: Into<Message>,
	{
		if let Some(id) = id {
			if !self.is_subscribed(id) {
				bail!("Not subscribed")
			}

			let queues = read!(self.queues);
			let sender = queues.get(&id).unwrap().sender.clone();

			sender.send(message.into())?;

			return Ok(());
		}

		let message: Message = message.into();
		let mut did_push = false;

		for listener in read!(self.listeners).iter() {
			let queues = read!(self.queues);
			let sender = queues.get(&listener.id).unwrap().sender.clone();

			sender.send(message.clone())?;

			did_push = true;
		}

		if !did_push {
			let max_unsynced_changes = Config::new().max_unsynced_changes;
			let mut unsynced_changes = write!(self.unsynced_changes);

			*unsynced_changes += 1;

			if max_unsynced_changes > 0 && *unsynced_changes >= max_unsynced_changes {
				argon_warn!(
					"There are {} unsynced changes. Connect at least one client to this server or increase max_unsynced_changes setting to suppress this warning",
					unsynced_changes.to_string().bold()
				);
			}
		}

		Ok(())
	}

	pub fn get(&self, id: u32) -> Result<Option<Message>> {
		if !self.is_subscribed(id) {
			bail!("Not subscribed")
		}

		let queues = read!(self.queues);
		let receiver = queues.get(&id).unwrap().receiver.clone();

		drop(queues);

		Ok(receiver.recv().ok())
	}

	pub fn get_timeout(&self, id: u32) -> Result<Option<Message>> {
		if !self.is_subscribed(id) {
			bail!("Not subscribed")
		}

		let queues = read!(self.queues);
		let receiver = queues.get(&id).unwrap().receiver.clone();

		drop(queues);

		Ok(receiver.recv_timeout(QUEUE_TIMEOUT).ok())
	}

	pub fn get_change(&self, id: u32) -> Result<Message> {
		if !self.is_subscribed(id) {
			bail!("Not subscribed")
		}

		let queues = read!(self.queues);
		let receiver = queues.get(&id).unwrap().receiver.clone();

		drop(queues);

		Ok(loop {
			if let Ok(message) = receiver.recv() {
				if message.is_change() {
					break message;
				}
			}
		})
	}

	pub fn subscribe(&self, id: u32, name: &str) -> Result<()> {
		if self.is_subscribed(id) {
			bail!("Already subscribed")
		}

		let (sender, receiver) = crossbeam_channel::unbounded();
		let channel = Channel { sender, receiver };

		let listener = Listener {
			id,
			name: name.to_owned(),
			is_internal: false,
		};

		write!(self.listeners).push(listener);
		write!(self.queues).insert(id.to_owned(), channel);

		Ok(())
	}

	pub fn subscribe_internal(&self) -> Result<()> {
		let mut id = 0;

		loop {
			if !self.is_subscribed(id) {
				break;
			}

			id += 1;
		}

		let (sender, receiver) = crossbeam_channel::unbounded();
		let channel = Channel { sender, receiver };

		let listener = Listener {
			id,
			name: format!("Internal listener #{}", id),
			is_internal: true,
		};

		write!(self.listeners).push(listener);
		write!(self.queues).insert(id.to_owned(), channel);

		Ok(())
	}

	pub fn unsubscribe(&self, id: u32) -> Result<()> {
		if !self.is_subscribed(id) {
			bail!("Not subscribed")
		}

		write!(self.listeners).retain(|listener| listener.id != id);
		write!(self.queues).remove(&id);

		Ok(())
	}

	pub fn disconnect(&self, message: &str, id: u32) -> Result<()> {
		if !self.is_subscribed(id) {
			bail!("Not subscribed")
		}

		self.push(
			server::Disconnect {
				message: message.to_owned(),
			},
			Some(id),
		)?;

		Ok(())
	}

	pub fn is_subscribed(&self, id: u32) -> bool {
		read!(self.listeners).iter().any(|listener| listener.id == id)
	}

	pub fn get_first_non_internal_listener_name(&self) -> Option<String> {
		read!(self.listeners)
			.iter()
			.find(|listener| !listener.is_internal)
			.map(|listener| listener.name.to_owned())
	}
}
