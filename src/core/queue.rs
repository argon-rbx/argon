use crate::messages::Message;

#[derive(Debug, Clone)]
struct QueueMessage {
	listeners: Vec<u64>,
	message: Message,
}

#[derive(Debug, Clone)]
pub struct Queue {
	queue: Vec<QueueMessage>,
	listeners: Vec<u64>,
}

impl Queue {
	pub fn new() -> Self {
		Self {
			queue: vec![],
			listeners: vec![],
		}
	}

	pub fn push(&mut self, message: Message) {
		let message = QueueMessage {
			listeners: self.listeners.clone(),
			message,
		};

		self.queue.push(message);

		println!("{:?}", self.queue);
	}

	pub fn pop(&mut self, id: u64) -> Option<Message> {
		if self.queue.is_empty() {
			return None;
		}

		let mut message_index: Option<usize> = None;

		for index in 0..self.queue.len() {
			let message = &mut self.queue[index];

			if message.listeners.contains(&id) {
				message.listeners.retain(|i| i != &id);

				if message.listeners.is_empty() {
					message_index = Some(index);
				}

				break;
			}
		}

		if let Some(index) = message_index {
			let message = self.queue.remove(index);

			return Some(message.message);
		}

		None
	}

	pub fn get(&mut self, id: u64) -> Option<Message> {
		if !self.is_subscribed(&id) {
			return None;
		}

		for message in &self.queue {
			if message.listeners.contains(&id) {
				return Some(message.message.clone());
			}
		}

		None
	}

	pub fn subscribe(&mut self, id: u64) -> bool {
		if self.is_subscribed(&id) {
			return false;
		}

		self.listeners.push(id);

		println!("{:?}", self.queue);

		true
	}

	pub fn unsubscribe(&mut self, id: u64) -> bool {
		if !self.is_subscribed(&id) {
			return false;
		}

		self.listeners.retain(|i| i != &id);

		let mut new_queue = vec![];

		for message in &mut self.queue {
			message.listeners.retain(|i| i != &id);

			if !message.listeners.is_empty() {
				new_queue.push(message.clone());
			}
		}

		self.queue = new_queue;

		println!("{:?}", self.queue);

		true
	}

	pub fn is_subscribed(&self, id: &u64) -> bool {
		self.listeners.contains(id)
	}
}
