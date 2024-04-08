use derive_from_one::FromOne;
use serde::Serialize;

use crate::{core::changes::Changes, project::ProjectDetails};

#[derive(Debug, Clone, Serialize, FromOne)]
pub enum Message {
	SyncChanges(SyncChanges),
	SyncDetails(SyncDetails),
	ExecuteCode(ExecuteCode),
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncChanges(pub Changes);

#[derive(Debug, Clone, Serialize)]
pub struct SyncDetails(pub ProjectDetails);

#[derive(Debug, Clone, Serialize)]
pub struct ExecuteCode {
	pub code: String,
}
