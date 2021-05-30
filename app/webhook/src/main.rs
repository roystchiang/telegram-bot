mod service;

use std::{env, net::SocketAddr, sync::Arc};

use common::CommonServer;
use common::Server;
use hyper::{
    service::{make_service_fn, service_fn},
    Server as HyperServer,
};
use telegram::{config::Settings, TelegramService};
use tokio::sync::RwLock;

use crate::service::WebhookSerice;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

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
    let common_server = Arc::new(CommonServer::new(WebhookSerice::new(arc_client)));

    let make_svc = make_service_fn(move |_conn| {
        let common_server = common_server.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let common_server = common_server.clone();
                async move { common_server.serve(req).await }
            }))
        }
    });

    let server = HyperServer::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
