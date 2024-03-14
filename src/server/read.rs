use actix_web::{
	get,
	web::{Data, Json, Query},
	HttpResponse, Responder,
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
async fn main(request: Query<Request>, core: Data<Arc<Core>>) -> impl Responder {
	let id = request.client_id;
	let queue = core.queue();

	if !queue.is_subscribed(id) {
		return HttpResponse::Unauthorized().body("Not subscribed");
	}

	match queue.get(id) {
		Ok(message) => HttpResponse::Ok().json(Json(message)),
		Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
	}
}
