use actix_msgpack::MsgPack;
use actix_web::{post, web::Data, HttpResponse, Responder};
use log::trace;
use std::sync::Arc;

use crate::{core::Core, server::AuthRequest};

#[post("/unsubscribe")]
async fn main(request: MsgPack<AuthRequest>, core: Data<Arc<Core>>) -> impl Responder {
	trace!("Received request: unsubscribe");

	let unsubscribed = core.queue().unsubscribe(request.client_id);

	if unsubscribed.is_ok() {
		HttpResponse::Ok().body("Unsubscribed successfully")
	} else {
		HttpResponse::BadRequest().body("Not subscribed")
	}
}
