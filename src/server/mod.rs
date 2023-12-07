use actix_web::{web, App, HttpServer, Responder};
use std::io::Result;

use crate::core::Core;

mod home;
mod stop;
mod sync_server;

async fn default_redirect() -> impl Responder {
	web::Redirect::to("/")
}

pub struct Server {
	core: Core,
	host: String,
	port: u16,
}

impl Server {
	pub fn new(core: Core, host: &String, port: &u16) -> Self {
		Self {
			core,
			host: host.to_owned(),
			port: port.to_owned(),
		}
	}

	#[actix_web::main]
	pub async fn start(&self) -> Result<()> {
		self.core.start();

		HttpServer::new(|| {
			App::new()
				.service(home::main)
				.service(stop::main)
				.service(sync_server::main)
				.default_service(web::to(default_redirect))
		})
		.bind((self.host.clone(), self.port))?
		.run()
		.await
	}
}
