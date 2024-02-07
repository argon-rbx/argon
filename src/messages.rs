use derive_from_one::FromOne;
use rbx_dom_weak::types::{Ref, Variant};
use serde::Serialize;
use std::collections::HashMap;

use crate::core::change::ModifiedSnapshot;

#[derive(Debug, Clone, Serialize, FromOne)]
pub enum Message {
	SyncMeta(SyncMeta),
	Execute(Execute),
	Create(Create),
	Remove(Remove),
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
	pub parent: Ref,
	pub name: String,
	pub class: String,
	pub properties: HashMap<String, Variant>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Remove {
	pub id: Ref,
}

#[derive(Debug, Clone, Serialize)]
pub struct Update(pub ModifiedSnapshot);
