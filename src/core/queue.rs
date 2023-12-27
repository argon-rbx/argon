use std::collections::{HashMap, VecDeque};

use crate::messages::Message;

#[derive(Debug, Clone)]
pub struct Queue {
	queues: HashMap<u64, VecDeque<Message>>,
	listeners: Vec<u64>,
}

impl Queue {
	pub fn new() -> Self {
		Self {
			queues: HashMap::new(),
			listeners: vec![],
		}
	}

	pub fn push(&mut self, message: Message, id: Option<&u64>) {
		if let Some(id) = id {
			if !self.is_subscribed(id) {
				return;
			}

			self.queues.get_mut(id).unwrap().push_back(message);

			return;
		}

		for listener in &self.listeners {
			self.queues.get_mut(listener).unwrap().push_back(message.clone());
		}
	}

	pub fn pop(&mut self, id: &u64) {
		if !self.is_subscribed(id) {
			return;
		}

		let queue = self.queues.get_mut(id).unwrap();

		if queue.is_empty() {
			return;
		}

		queue.pop_front();
	}

	pub fn get(&mut self, id: &u64) -> Option<&Message> {
		if !self.is_subscribed(id) {
			return None;
		}

		self.queues.get(id).unwrap().get(0)
	}

	pub fn subscribe(&mut self, id: &u64) -> bool {
		if self.is_subscribed(id) {
			return false;
		}

		self.listeners.push(id.to_owned());
		self.queues.insert(id.to_owned(), VecDeque::new());

		true
	}

	pub fn unsubscribe(&mut self, id: &u64) -> bool {
		if !self.is_subscribed(id) {
			return false;
		}

		self.listeners.retain(|i| i != id);
		self.queues.remove(id);

		true
	}

	pub fn is_subscribed(&self, id: &u64) -> bool {
		self.listeners.contains(id)
	}
}
