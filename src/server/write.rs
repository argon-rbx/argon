use actix_msgpack::MsgPack;
use actix_web::{post, web::Data, HttpResponse, Responder};
use log::trace;
use std::sync::Arc;

use crate::core::{processor::WriteRequest, Core};

#[post("/write")]
async fn main(request: MsgPack<WriteRequest>, core: Data<Arc<Core>>) -> impl Responder {
	trace!("Received request: write");

	let request = request.0;

	if !core.queue().is_subscribed(request.client_id) {
		return HttpResponse::Unauthorized().body("Not subscribed");
	}

	core.processor().write(request);

	HttpResponse::Ok().body("Written changes successfully")
}
