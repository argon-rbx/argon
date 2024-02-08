use actix_web::{
	post,
	web::{Data, Json},
	HttpResponse, Responder,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{core::Core, messages};

#[derive(Deserialize, Debug)]
struct Request {
	code: String,
}

#[post("/exec")]
async fn main(request: Json<Request>, core: Data<Arc<Core>>) -> impl Responder {
	core.queue().push(
		messages::Execute {
			code: request.code.clone(),
		},
		None,
	);

	HttpResponse::Ok().body("Code executed successfully")
}
