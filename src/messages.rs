use rbx_dom_weak::types::Variant;
use serde::Serialize;
use std::collections::HashMap;

use crate::rbx_path::RbxPath;

#[derive(Debug, Clone, Serialize)]
pub enum Message {
	SyncMeta(SyncMeta),
	Execute(Execute),
	Create(Create),
	Delete(Delete),
	Update(Update),
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncMeta {
	pub name: String,
	pub game_id: Option<u64>,
	pub place_ids: Option<Vec<u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Execute {
	pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Create {
	pub class: String,
	pub path: RbxPath,
	pub properties: HashMap<String, Variant>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Delete {
	pub path: RbxPath,
}

#[derive(Debug, Clone, Serialize)]
pub struct Update {
	pub path: RbxPath,
	pub properties: HashMap<String, Variant>,
}
