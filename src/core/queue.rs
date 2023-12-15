use std::collections::HashMap;

use crate::messages::Message;

#[derive(Debug, Clone)]
pub struct Queue {
	queues: HashMap<u64, Vec<Message>>,
	listeners: Vec<u64>,
}

impl Queue {
	pub fn new() -> Self {
		Self {
			queues: HashMap::new(),
			listeners: vec![],
		}
	}

	pub fn push(&mut self, message: Message) {
		for id in &self.listeners {
			self.queues.get_mut(id).unwrap().push(message.clone());
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

		queue.remove(0);
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
		self.queues.insert(id.to_owned(), vec![]);

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
