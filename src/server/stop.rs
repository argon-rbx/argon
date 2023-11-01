use actix_web::{post, HttpResponse, Responder};
use std::{process, thread, time::Duration};

use crate::argon_info;

async fn stop() {
	argon_info!("Argon stopped!");
	thread::sleep(Duration::from_millis(1000));
	process::exit(1);
}

#[post("stop")]
async fn main() -> impl Responder {
	tokio::spawn(stop());
	HttpResponse::Ok().finish()
}
