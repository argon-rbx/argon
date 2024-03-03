use derive_from_one::FromOne;
use serde::Serialize;

use crate::core::changes::{AddedSnapshot, ModifiedSnapshot, RemovedSnapshot};

#[derive(Debug, Clone, Serialize, FromOne)]
pub enum Message {
	SyncDetails(SyncDetails),
	Execute(Execute),
	Create(Create),
	Remove(Remove),
	Update(Update),
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncDetails {
	pub name: String,
	pub game_id: Option<u64>,
	pub place_ids: Option<Vec<u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Execute {
	pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Create(pub AddedSnapshot);

#[derive(Debug, Clone, Serialize)]
pub struct Remove(pub RemovedSnapshot);

#[derive(Debug, Clone, Serialize)]
pub struct Update(pub ModifiedSnapshot);
