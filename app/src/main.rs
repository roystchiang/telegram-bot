mod config;
mod service;
pub mod telegram;

use std::{env, net::SocketAddr, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use tokio::sync::RwLock;

use crate::{config::Settings, service::handler, telegram::TelegramService};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[tokio::main]
async fn main() {
    // Setup
    pretty_env_logger::init();
    let args: Vec<String> = env::args().collect();
    let settings = Settings::new(&args[1]).expect("Unable to load config");

    info!("Starting server");
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let client = TelegramService::new(&settings.telegram.api_key);
    let arc_client = Arc::new(RwLock::new(client));

    let make_svc = make_service_fn(move |_conn| {
        let arc_clone = arc_client.clone();
        async move { Ok::<_, hyper::Error>(service_fn(move |req| handler(req, arc_clone.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
