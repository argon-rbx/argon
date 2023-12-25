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
}

#[post("/read_all")]
async fn main(request: Json<Request>, core: Data<Arc<Core>>) -> impl Responder {
	let id = request.client_id;
	let queue = core.queue();

	if !queue.is_subscribed(&id) {
		return HttpResponse::BadRequest().body("Not subscribed");
	}

	drop(queue);
	core.sync_dom(id);

	HttpResponse::Ok().body("Started syncing DOM")
}
