use actix_web::{
	post,
	web::{Data, Json},
	HttpResponse, Responder,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{core::Core, messages};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
	code: String,
}

#[post("/exec")]
async fn main(request: Json<Request>, core: Data<Arc<Core>>) -> impl Responder {
	let pushed = core.queue().push(
		messages::ExecuteCode {
			code: request.code.clone(),
		},
		None,
	);

	match pushed {
		Ok(()) => HttpResponse::Ok().body("Code executed successfully"),
		Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
	}
}
