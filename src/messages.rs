use rbx_dom_weak::types::Variant;
use serde::Serialize;
use std::collections::HashMap;

use crate::rbx_path::RbxPath;

#[derive(Debug, Clone, Serialize)]
pub enum Message {
	SyncMeta(SyncMeta),
	Create(Create),
	Delete(Delete),
	Write(Update),
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncMeta {
	pub name: String,
	pub game_id: Option<u64>,
	pub place_ids: Option<Vec<u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Create {
	pub name: String,
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
