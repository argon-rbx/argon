use actix_web::{post, HttpResponse, Responder};
use log::trace;
use std::process;

async fn stop() {
	trace!("Stopping Argon!");
	process::exit(0);
}

#[post("stop")]
async fn main() -> impl Responder {
	tokio::spawn(stop());
	HttpResponse::Ok().body("Argon stopped")
}
