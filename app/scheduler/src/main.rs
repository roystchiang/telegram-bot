use std::{net::SocketAddr, sync::Arc};
mod service;

use common::CommonServer;
use common::Server;
use hyper::{
    service::{make_service_fn, service_fn},
    Server as HyperServer,
};
use storage::sled::SledKeyValue;

use crate::service::SchedulerService;

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    // Setup
    pretty_env_logger::init();
    info!("Starting server");

    let storage = match SledKeyValue::new("data") {
        Ok(storage) => storage,
        Err(e) => {
            error!("Failed to initialize sled storage: {}", e);
            panic!()
        }
    };
    let arc_storage = Arc::new(storage);
    let common_server = Arc::new(CommonServer::new(SchedulerService::new(arc_storage)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let make_svc = make_service_fn(move |_conn| {
        let common_server = common_server.clone();
        async move {
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
