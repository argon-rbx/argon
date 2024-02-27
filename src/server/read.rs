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
struct Response {
	queue: Vec<Message>,
}

#[get("/read")]
async fn main(request: Query<Request>, core: Data<Arc<Core>>) -> Result<impl Responder> {
	let id = request.client_id;
	let mut queue = core.queue();

	if !queue.is_subscribed(&id) {
		return Err(error::ErrorBadRequest("Not subscribed"));
	}

	let mut new_queue = vec![];

	loop {
		let message = queue.get(&id);

		if let Some(message) = message {
			new_queue.push(message.clone());
			queue.pop(&id);
		} else {
			break;
		}
	}

	let response = Response { queue: new_queue };

	Ok(Json(response))
}
