pub mod telegram;
mod config;

use std::{env, net::SocketAddr, sync::Arc};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

use hyper::{Body, Request, Response, Server, body::Buf, service::{make_service_fn, service_fn}};
use tokio::sync::{RwLock};

use crate::{config::Settings, telegram::{Telegram, Update}};

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
    let client = Telegram::new(
        &settings.telegram.api_key
    );
    let arc_client = Arc::new(RwLock::new(client));

    let make_svc = make_service_fn(move |_conn| {
        let arc_clone = arc_client.clone();
        async move { Ok::<_, GenericError>(service_fn(move |req| hello_world(req, arc_clone.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    println!("Sup dude");
}

async fn hello_world(
    req: Request<Body>,
    telegram: Arc<RwLock<telegram::Telegram>>,
) -> Result<Response<Body>> {
    info!(
        "path: {}, method: {}, query: {:?}",
        req.uri().path(),
        req.method(),
        req.uri().query()
    );
    let body = hyper::body::aggregate(req).await?;

    // try to parse as json with serde_json
    match serde_json::from_reader::<_, Update>(body.reader()) {
        Ok(update) => {
            info!("Update: {:?}", update);
            let client = telegram.read().await;
            client.send_message(update.message.chat.id, "ack").await;

        }
        Err(e) => {
            error!("error: {}", e)
        }
    }
    Ok(Response::new("Ok".into()))
}
