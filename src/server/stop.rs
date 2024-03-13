use actix_web::{post, HttpResponse, Responder};
use log::trace;
use std::process;

use crate::util;

async fn stop() {
	trace!("Stopping Argon!");
	// We need to kill all child processes as well
	util::kill_process(process::id());
}

#[post("/stop")]
async fn main() -> impl Responder {
	tokio::spawn(stop());
	HttpResponse::Ok().body("Argon stopped successfully")
}
