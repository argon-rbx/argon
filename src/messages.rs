use serde::Serialize;

use crate::types::{RobloxKind, RobloxPath};

#[derive(Debug, Clone, Serialize)]
pub enum Message {
	UpdateMeta(UpdateMeta),
	Sync(Sync),
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateMeta {
	pub name: String,
	pub game_id: Option<u64>,
	pub place_ids: Option<Vec<u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sync {
	pub action: SyncAction,
	pub path: RobloxPath,
	pub kind: Option<RobloxKind>,
	pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum SyncAction {
	Create,
	Update,
	Delete,
	Write,
}