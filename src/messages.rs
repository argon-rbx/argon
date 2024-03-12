use derive_from_one::FromOne;
use serde::Serialize;

use crate::core::changes::Changes;

#[derive(Debug, Clone, Serialize, FromOne)]
pub enum Message {
	SyncChanges(SyncChanges),
	SyncDetails(SyncDetails),
	ExecuteCode(ExecuteCode),
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncChanges(pub Changes);

#[derive(Debug, Clone, Serialize)]
pub struct SyncDetails {
	pub name: String,
	pub game_id: Option<u64>,
	pub place_ids: Option<Vec<u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecuteCode {
	pub code: String,
}
