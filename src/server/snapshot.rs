use actix_msgpack::MsgPackResponseBuilder;
use actix_web::{get, web::Data, HttpResponse, Responder};
use std::sync::Arc;

use crate::core::Core;

#[get("/snapshot")]
async fn main(core: Data<Arc<Core>>) -> impl Responder {
	HttpResponse::Ok().msgpack(core.snapshot())
}
