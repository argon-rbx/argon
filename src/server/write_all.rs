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
	snapshot: Snapshot,
}

#[derive(Serialize)]
struct Response(Snapshot);

#[get("/writeAll")]
async fn main(request: Query<Request>, core: Data<Arc<Core>>) -> impl Responder {
	if !core.queue().is_subscribed(request.client_id) {
		return HttpResponse::Unauthorized().body("Not subscribed");
	}

	core.processor().write_all(request.snapshot.clone());

	HttpResponse::Ok().body("Written all changes successfully")
}
