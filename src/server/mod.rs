use actix_web::{
	web::{self, Data},
	App, HttpServer, Responder,
};
use std::{io::Result, net::TcpStream, sync::Arc};

use crate::core::Core;

mod details;
mod exec;
mod home;
mod read;
mod snapshot;
mod stop;
mod subscribe;
mod unsubscribe;

async fn default_redirect() -> impl Responder {
	web::Redirect::to("/")
}

pub struct Server {
	core: Arc<Core>,
	host: String,
	port: u16,
}

impl Server {
	pub fn new(core: Arc<Core>, host: &String, port: &u16) -> Self {
		Self {
			core,
			host: host.to_owned(),
			port: port.to_owned(),
		}
	}

	#[actix_web::main]
	pub async fn start(&self) -> Result<()> {
		let core = self.core.clone();

		HttpServer::new(move || {
			App::new()
				.app_data(Data::new(core.clone()))
				.service(details::main)
				.service(subscribe::main)
				.service(unsubscribe::main)
				.service(home::main)
				.service(stop::main)
				.service(read::main)
				.service(snapshot::main)
				.service(exec::main)
				.default_service(web::to(default_redirect))
		})
		.bind((self.host.clone(), self.port))?
		.run()
		.await
	}
}

pub fn is_port_free(host: &str, port: u16) -> bool {
	TcpStream::connect((host, port)).is_err()
}

pub fn get_free_port(host: &str, port: u16) -> u16 {
	let mut port = port;

	while !is_port_free(host, port) {
		port += 1;
	}

	port
}
