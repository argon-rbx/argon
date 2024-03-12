use actix_web::{
	error, get,
	web::{Data, Json, Query},
	Responder, Result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::{snapshot::Snapshot, Core};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	client_id: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Response(Snapshot);

#[get("/snapshot")]
async fn main(request: Query<Request>, core: Data<Arc<Core>>) -> Result<impl Responder> {
	let id = request.client_id;

	if !core.queue().is_subscribed(&id) {
		return Err(error::ErrorBadRequest("Not subscribed"));
	}

	Ok(Json(core.snapshot()))
}
