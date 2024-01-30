use rbx_dom_weak::types::{Ref, Variant};
use std::collections::HashMap;

use super::snapshot::Snapshot;

#[derive(Debug)]
pub struct ModifiedSnapshot {
	pub id: Ref,
	pub name: Option<String>,
	pub class: Option<String>,
	pub properties: Option<HashMap<String, Variant>>,
}

impl ModifiedSnapshot {
	pub fn new(id: Ref) -> Self {
		Self {
			id,
			name: None,
			class: None,
			properties: None,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.name.is_none() && self.class.is_none() && self.properties.is_none()
	}
}

#[derive(Debug)]
pub struct Changes {
	pub additions: Vec<Snapshot>,
	pub modifications: Vec<ModifiedSnapshot>,
	pub removals: Vec<Ref>,
}

impl Changes {
	pub fn new() -> Self {
		Self {
			additions: Vec::new(),
			modifications: Vec::new(),
			removals: Vec::new(),
		}
	}

	pub fn add(&mut self, snapshot: Snapshot) {
		self.additions.push(snapshot);
	}

	pub fn modify(&mut self, modified_snapshot: ModifiedSnapshot) {
		self.modifications.push(modified_snapshot);
	}

	pub fn remove(&mut self, id: Ref) {
		self.removals.push(id);
	}

	pub fn extend(&mut self, changes: Self) {
		self.additions.extend(changes.additions);
		self.modifications.extend(changes.modifications);
		self.removals.extend(changes.removals);
	}

	pub fn is_empty(&self) -> bool {
		self.additions.is_empty() && self.modifications.is_empty() && self.removals.is_empty()
	}
}
