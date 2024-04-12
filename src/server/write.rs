use actix_msgpack::MsgPack;
use actix_web::{post, web::Data, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::core::{changes::Changes, Core};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	client_id: u32,
	changes: Changes,
}

#[post("/write")]
async fn main(request: MsgPack<Request>, core: Data<Arc<Core>>) -> impl Responder {
	if !core.queue().is_subscribed(request.client_id) {
		return HttpResponse::Unauthorized().body("Not subscribed");
	}

	core.processor().write(request.changes.clone());

	HttpResponse::Ok().body("Written changes successfully")
}
