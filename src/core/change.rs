use rbx_dom_weak::types::Ref;

use super::snapshot::Snapshot;

#[derive(Debug)]
pub struct ModifiedSnapshot {
	pub id: Ref,
	pub name: Option<String>,
	pub class: Option<String>,
	pub properties: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ChangeList {
	pub added: Vec<Snapshot>,
	pub modified: Vec<ModifiedSnapshot>,
	pub removed: Vec<Ref>,
}

impl ChangeList {
	fn new() -> Self {
		Self {
			added: Vec::new(),
			modified: Vec::new(),
			removed: Vec::new(),
		}
	}

	fn add(&mut self, snapshot: Snapshot) {
		self.added.push(snapshot);
	}

	fn modify(&mut self, modified_snapshot: ModifiedSnapshot) {
		self.modified.push(modified_snapshot);
	}

	fn remove(&mut self, id: Ref) {
		self.removed.push(id);
	}
}
