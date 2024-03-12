use log::error;
use rbx_dom_weak::types::{Ref, Variant};
use serde::Serialize;
use std::collections::HashMap;

use super::snapshot::Snapshot;

#[derive(Debug, Clone, Serialize)]
pub struct AddedSnapshot {
	pub id: Ref,
	pub parent: Ref,
	pub name: String,
	pub class: String,
	pub properties: HashMap<String, Variant>,
	pub children: Vec<Snapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdatedSnapshot {
	pub id: Ref,
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

	pub fn add(&mut self, mut snapshot: Snapshot, parent: Ref) {
		let id = if let Some(id) = snapshot.id {
			id
		} else {
			error!("Attempted to add a snapshot without an ID to changes: {:?}", snapshot);
			return;
		};

		snapshot.meta = None;
		snapshot.paths.clear();
		snapshot.file_type = None;

		self.additions.push(AddedSnapshot {
			id,
			parent,
			name: snapshot.name,
			class: snapshot.class,
			properties: snapshot.properties,
			children: snapshot.children,
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
}
