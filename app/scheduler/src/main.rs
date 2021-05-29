
use std::{net::SocketAddr, sync::Arc};

use hyper::{Body, Request, Response, Server, service::{make_service_fn, service_fn}};
use storage::{KeyValue, sled::SledKeyValue};

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

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let make_svc = make_service_fn(|_conn| {
        let storage_clone = arc_storage.clone();
        async move { 
            Ok::<_, hyper::Error>(service_fn(move |req| handler(req, storage_clone.clone())))
        }
    });


    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handler(
    _req: Request<Body>,
    _storage: Arc<impl KeyValue>,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Response::new("done".into()))
}