use actix_msgpack::MsgPack;
use actix_web::{post, web::Data, HttpResponse, Responder};
use log::trace;
use serde::Deserialize;
use std::sync::Arc;

use crate::core::Core;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	client_id: u32,
	name: String,
}

#[post("/subscribe")]
async fn main(request: MsgPack<Request>, core: Data<Arc<Core>>) -> impl Responder {
	trace!("Received request: subscribe");

	let subscribed = core.queue().subscribe(request.client_id, &request.name);

	if subscribed.is_ok() {
		HttpResponse::Ok().body("Subscribed successfully")
	} else {
		HttpResponse::BadRequest().body("Already subscribed")
	}
}
