use actix_web::{
	get,
	web::{Data, Query},
	HttpResponse, Responder,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::core::Core;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	client_id: u64,
}

#[get("/read_all")]
async fn main(request: Query<Request>, core: Data<Arc<Core>>) -> impl Responder {
	let id = request.client_id;

	if !core.queue().is_subscribed(&id) {
		return HttpResponse::BadRequest().body("Not subscribed");
	}

	// core.sync_dom(id);

	HttpResponse::Ok().body("Started syncing DOM")
}
