use actix_msgpack::MsgPack;
use actix_web::{post, web::Data, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::core::Core;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	client_id: u32,
}

#[post("/subscribe")]
async fn main(request: MsgPack<Request>, core: Data<Arc<Core>>) -> impl Responder {
	let subscribed = core.queue().subscribe(request.client_id);

	if subscribed.is_ok() {
		HttpResponse::Ok().body("Subscribed successfully")
	} else {
		HttpResponse::BadRequest().body("Already subscribed")
	}
}
