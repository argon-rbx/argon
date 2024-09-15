use actix_msgpack::{MsgPack, MsgPackResponseBuilder};
use actix_web::{post, web::Data, HttpResponse, Responder};
use log::trace;
use rbx_dom_weak::types::Ref;
use serde::Deserialize;
use std::sync::Arc;

use crate::core::Core;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	instance: Ref,
}

#[post("/snapshot")]
async fn main(request: MsgPack<Request>, core: Data<Arc<Core>>) -> impl Responder {
	trace!("Received request: snapshot");
	HttpResponse::Ok().msgpack(core.snapshot(request.instance))
}
