use actix_web::{post, HttpResponse, Responder};
use log::{info, trace};
use std::process;

use argon::util;
// use argon::{core::Core, ext::ResultExt, server::AuthRequest}; // This line was added incorrectly and will be removed

#[post("/stop")]
async fn main() -> impl Responder {
	trace!("Received request: stop");
	info!("Stopping Argon!");

	util::kill_process(process::id());

	HttpResponse::Ok().body("Argon stopped successfully")
}
