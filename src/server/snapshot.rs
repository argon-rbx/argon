use actix_msgpack::MsgPackResponseBuilder;
use actix_web::{get, web::Data, HttpResponse, Responder};
use serde::Serialize;
use std::sync::Arc;

use crate::core::{snapshot::Snapshot, Core};

#[derive(Serialize)]
struct Response(Snapshot);

#[get("/snapshot")]
async fn main(core: Data<Arc<Core>>) -> impl Responder {
	HttpResponse::Ok().msgpack(core.snapshot())
}
