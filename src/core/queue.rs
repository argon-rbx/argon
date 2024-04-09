use anyhow::{bail, Result};
use crossbeam_channel::{Receiver, Sender};
use std::{collections::HashMap, sync::RwLock, time::Duration};

use crate::messages::Message;

const TIMEOUT: Duration = Duration::from_secs(60);

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

#[derive(Debug)]
struct Channel {
	sender: Sender<Message>,
	receiver: Receiver<Message>,
}

#[derive(Debug)]
pub struct Queue {
	queues: RwLock<HashMap<u32, Channel>>,
	listeners: RwLock<Vec<u32>>,
}

impl Queue {
	pub fn new() -> Self {
		Self {
			queues: RwLock::new(HashMap::new()),
			listeners: RwLock::new(vec![]),
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

		for listener in read!(self.listeners).iter() {
			let queues = read!(self.queues);
			let sender = queues.get(listener).unwrap().sender.clone();

			sender.send(message.clone())?;
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

		let message = receiver.recv_timeout(TIMEOUT).ok();

		Ok(message)
	}

	pub fn subscribe(&self, id: u32) -> Result<()> {
		if self.is_subscribed(id) {
			bail!("Already subscribed")
		}

		let (sender, receiver) = crossbeam_channel::unbounded();
		let channel = Channel { sender, receiver };

		write!(self.listeners).push(id.to_owned());
		write!(self.queues).insert(id.to_owned(), channel);

		Ok(())
	}

	pub fn unsubscribe(&self, id: u32) -> Result<()> {
		if !self.is_subscribed(id) {
			bail!("Not subscribed")
		}

		write!(self.listeners).retain(|i| i != &id);
		write!(self.queues).remove(&id);

		Ok(())
	}

	pub fn is_subscribed(&self, id: u32) -> bool {
		read!(self.listeners).contains(&id)
	}
}
