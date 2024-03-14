use actix_web::{
	get,
	web::{Data, Json},
	HttpResponse, Responder,
};
use serde::Serialize;
use std::sync::Arc;

use crate::core::{snapshot::Snapshot, Core};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Response(Snapshot);

#[get("/snapshot")]
async fn main(core: Data<Arc<Core>>) -> impl Responder {
	HttpResponse::Ok().json(Json(core.snapshot()))
}
