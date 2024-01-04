use actix_web::{post, HttpResponse, Responder};
use log::trace;
use std::process;

use crate::util;

async fn stop() {
	trace!("Stopping Argon!");
	util::kill(process::id());
}

#[post("stop")]
async fn main() -> impl Responder {
	tokio::spawn(stop());
	HttpResponse::Ok().body("Argon stopped")
}
