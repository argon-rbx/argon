use actix_msgpack::MsgPackResponseBuilder;
use actix_web::{
	get,
	web::{Data, Query},
	HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::{snapshot::Snapshot, Core};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	client_id: u32,
}

#[derive(Serialize)]
struct Response(Snapshot);

#[get("/readAll")]
async fn main(request: Query<Request>, core: Data<Arc<Core>>) -> impl Responder {
	if !core.queue().is_subscribed(request.client_id) {
		return HttpResponse::Unauthorized().body("Not subscribed");
	}

	HttpResponse::Ok().msgpack(core.snapshot())
}
