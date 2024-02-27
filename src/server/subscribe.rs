use actix_web::{
	post,
	web::{Data, Json},
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

#[post("/subscribe")]
async fn main(request: Json<Request>, core: Data<Arc<Core>>) -> impl Responder {
	let subscribed = core.queue().subscribe(&request.client_id);

	if subscribed {
		HttpResponse::Ok().body("Subscribed successfully")
	} else {
		HttpResponse::BadRequest().body("Already subscribed")
	}
}
