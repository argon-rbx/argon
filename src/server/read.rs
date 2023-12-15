use actix_web::{
	error, get,
	web::{Data, Json, Query},
	Responder, Result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{core::Core, messages::Message};

#[derive(Deserialize, Debug)]
struct Request {
	client_id: u64,
}

#[derive(Serialize)]
struct Response {
	changes: Vec<Message>,
}

#[get("/read")]
async fn main(request: Query<Request>, core: Data<Arc<Core>>) -> Result<impl Responder> {
	let id = request.client_id;
	let mut queue = core.queue();

	if !queue.is_subscribed(&id) {
		return Err(error::ErrorBadRequest("Not subscribed"));
	}

	let mut changes = vec![];

	loop {
		let message = queue.get(&id);

		if let Some(message) = message {
			changes.push(message.clone());
			queue.pop(&id);
		} else {
			break;
		}
	}

	let response = Response { changes };

	Ok(Json(response))
}
