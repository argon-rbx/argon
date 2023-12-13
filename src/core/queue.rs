use crate::messages::Message;

pub struct Queue {
	queue: Vec<Message>,
	subcribers: Vec<u64>,
}

impl Queue {
	pub fn new() -> Self {
		Self {
			queue: vec![],
			subcribers: vec![],
		}
	}

	pub fn push(&mut self, message: Message) {
		self.queue.push(message);

		println!("{:?}", self.queue);
	}

	pub fn subscribe(&mut self, id: u64) -> bool {
		if self.is_subscribed(&id) {
			return false;
		}

		self.subcribers.push(id);

		true
	}

	pub fn unsubscribe(&mut self, id: u64) -> bool {
		if !self.is_subscribed(&id) {
			return false;
		}

		self.subcribers.retain(|subcriber| subcriber != &id);

		true
	}

	pub fn is_subscribed(&self, id: &u64) -> bool {
		self.subcribers.contains(id)
	}
}
