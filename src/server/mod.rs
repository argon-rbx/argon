use actix_web::{
	web::{self, Data},
	App, HttpServer, Responder,
};
use std::{io::Result, net::TcpListener, sync::Arc};

use crate::core::Core;

mod details;
mod exec;
mod home;
mod open;
mod read;
mod snapshot;
mod stop;
mod subscribe;
mod unsubscribe;
mod write;

async fn default_redirect() -> impl Responder {
	web::Redirect::to("/")
}

pub struct Server {
	core: Arc<Core>,
	host: String,
	port: u16,
}

impl Server {
	pub fn new(core: Arc<Core>, host: &str, port: u16) -> Self {
		Self {
			core,
			host: host.to_owned(),
			port,
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
				.service(snapshot::main)
				.service(read::main)
				.service(write::main)
				.service(exec::main)
				.service(open::main)
				.service(stop::main)
				.service(home::main)
				.default_service(web::to(default_redirect))
		})
		.backlog(0)
		.bind((self.host.clone(), self.port))?
		.run()
		.await
	}
}

pub fn is_port_free(host: &str, port: u16) -> bool {
	TcpListener::bind((host, port)).is_ok()
}

pub fn get_free_port(host: &str, port: u16) -> u16 {
	let mut port = port;

	while !is_port_free(host, port) {
		port += 1;
	}

	port
}

pub fn format_address(host: &str, port: u16) -> String {
	format!("http://{}:{}", host, port)
}
