use actix_msgpack::MsgPackResponseBuilder;
use actix_web::{get, web::Data, HttpResponse, Responder};
use std::sync::Arc;

use crate::{core::Core, project::ProjectDetails};

#[get("/details")]
async fn main(core: Data<Arc<Core>>) -> impl Responder {
	HttpResponse::Ok().msgpack(ProjectDetails::from_project(&core.project(), &core.tree()))
}
