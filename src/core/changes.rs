use rbx_dom_weak::types::{Ref, Variant};
use serde::Serialize;
use std::collections::HashMap;

use super::{meta::Meta, snapshot::Snapshot};

#[derive(Debug, Clone, Serialize)]
pub struct AddedSnapshot {
	pub id: Ref,
	pub meta: Meta,
	pub parent: Ref,
	pub name: String,
	pub class: String,
	pub properties: HashMap<String, Variant>,
	pub children: Vec<Snapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdatedSnapshot {
	pub id: Ref,
	pub meta: Option<Meta>,
	pub name: Option<String>,
	pub class: Option<String>,
	pub properties: Option<HashMap<String, Variant>>,
}

impl UpdatedSnapshot {
	pub fn new(id: Ref) -> Self {
		Self {
			id,
			name: None,
			class: None,
			properties: None,
			meta: None,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.name.is_none() && self.class.is_none() && self.properties.is_none()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct Changes {
	pub additions: Vec<AddedSnapshot>,
	pub updates: Vec<UpdatedSnapshot>,
	pub removals: Vec<Ref>,
}

impl Changes {
	pub fn new() -> Self {
		Self {
			additions: Vec::new(),
			updates: Vec::new(),
			removals: Vec::new(),
		}
	}

	pub fn add(&mut self, snapshot: Snapshot, parent: Ref) {
		self.additions.push(AddedSnapshot {
			id: snapshot.id,
			parent,
			name: snapshot.name,
			class: snapshot.class,
			properties: snapshot.properties,
			children: snapshot.children,
			meta: snapshot.meta,
		});
	}

	pub fn update(&mut self, modified_snapshot: UpdatedSnapshot) {
		self.updates.push(modified_snapshot);
	}

	pub fn remove(&mut self, id: Ref) {
		self.removals.push(id);
	}

	pub fn extend(&mut self, changes: Self) {
		self.additions.extend(changes.additions);
		self.updates.extend(changes.updates);
		self.removals.extend(changes.removals);
	}

	pub fn is_empty(&self) -> bool {
		self.additions.is_empty() && self.updates.is_empty() && self.removals.is_empty()
	}

	pub fn len(&self) -> usize {
		self.additions.len() + self.updates.len() + self.removals.len()
	}
}
