use actix_web::{
	error, get,
	web::{Data, Json, Query},
	Responder, Result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{core::Core, messages::Message};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	client_id: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Response(Option<Message>);

#[get("/read")]
async fn main(request: Query<Request>, core: Data<Arc<Core>>) -> Result<impl Responder> {
	let id = request.client_id;
	let queue = core.queue();

	if !queue.is_subscribed(&id) {
		return Err(error::ErrorBadRequest("Not subscribed"));
	}

	match queue.get(&id) {
		Ok(message) => Ok(Json(message)),
		Err(err) => Err(error::ErrorInternalServerError(err)),
	}
}
