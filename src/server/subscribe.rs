use actix_msgpack::MsgPack;
use actix_web::{post, web::Data, HttpResponse, Responder};
use std::sync::Arc;

use crate::{core::Core, server::AuthRequest};

#[post("/subscribe")]
async fn main(request: MsgPack<AuthRequest>, core: Data<Arc<Core>>) -> impl Responder {
	let subscribed = core.queue().subscribe(request.client_id);

	if subscribed.is_ok() {
		HttpResponse::Ok().body("Subscribed successfully")
	} else {
		HttpResponse::BadRequest().body("Already subscribed")
	}
}
