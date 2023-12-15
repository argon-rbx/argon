use actix_web::{
	post,
	web::{Data, Json},
	HttpResponse, Responder,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::core::Core;

#[derive(Deserialize, Debug)]
struct Request {
	client_id: u64,
	place_id: u64,
	game_id: u64,
}

#[post("/subscribe")]
async fn main(request: Json<Request>, core: Data<Arc<Core>>) -> impl Responder {
	if let Some(game_id) = core.game_id() {
		if game_id != request.game_id {
			return HttpResponse::BadRequest().body("game_id mismatch");
		}
	}

	if let Some(place_ids) = core.place_ids() {
		if !place_ids.contains(&request.place_id) {
			return HttpResponse::BadRequest().body("place_id mismatch");
		}
	}

	let subscribed = core.queue().subscribe(&request.client_id);

	if subscribed {
		HttpResponse::Ok().body("Subscribed successfully")
	} else {
		HttpResponse::BadRequest().body("Already subscribed")
	}
}
