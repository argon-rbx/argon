use derive_from_one::FromOne;
use serde::Serialize;

use crate::core::changes::{AddedSnapshot, RemovedSnapshot, UpdatedSnapshot};

#[derive(Debug, Clone, Serialize, FromOne)]
pub enum Message {
	// Syncing changes
	Add(Add),
	Remove(Remove),
	Update(Update),

	// Misc
	SyncDetails(SyncDetails),
	Execute(Execute),
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
pub struct Add(pub AddedSnapshot);

#[derive(Debug, Clone, Serialize)]
pub struct Remove(pub RemovedSnapshot);

#[derive(Debug, Clone, Serialize)]
pub struct Update(pub UpdatedSnapshot);
