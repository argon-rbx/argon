use actix_msgpack::MsgPackResponseBuilder;
use actix_web::{
	get,
	web::{Data, Query},
	HttpResponse, Responder,
};
use log::trace;
use std::sync::Arc;

use crate::{core::Core, server::AuthRequest};

#[get("/read")]
async fn main(request: Query<AuthRequest>, core: Data<Arc<Core>>) -> impl Responder {
	trace!("Received request: read");

	let id = request.client_id;
	let queue = core.queue();

	if !queue.is_subscribed(id) {
		return HttpResponse::Unauthorized().body("Not subscribed");
	}

	match queue.get_timeout(id) {
		Ok(message) => HttpResponse::Ok().msgpack(message),
		Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
	}
}
