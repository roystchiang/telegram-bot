use std::{net::SocketAddr, sync::Arc};

use hyper::{
    body::{aggregate, Buf},
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use scheduler::inputs::SchedulerUpdate;
use storage::{sled::SledKeyValue, KeyValue};

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
        async move { Ok::<_, hyper::Error>(service_fn(move |req| handler(req, storage_clone.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handler(
    req: Request<Body>,
    storage: Arc<impl KeyValue>,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let body = aggregate(req).await?;
    let data = serde_json::from_reader::<_, SchedulerUpdate>(body.reader())?;

    storage
        .set(
            data.update.update_id.to_string(),
            data.update
                .message
                .text
                .unwrap_or("empty message".to_string()),
        )
        .await?;
    println!("{:?}", data.update.update_id.to_string());
    Ok(Response::new("done".into()))
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use hyper::{Request, StatusCode};
    use scheduler::inputs::SchedulerUpdate;
    use storage::sled::SledKeyValue;
    use storage::KeyValue;
    use telegram::types::{Chat, Message, Update};
    use tempfile::TempDir;

    use crate::handler;

    #[tokio::test]
    async fn should_store_received_message_update() {
        let db = get_db();
        let input = SchedulerUpdate {
            update: Update {
                update_id: 1,
                message: Message {
                    message_id: 2,
                    chat: Chat { id: 3 },
                    text: Some("message".to_string()),
                    entities: None,
                },
            },
        };
        let request = Request::builder()
            .body(serde_json::to_string(&input).unwrap().into())
            .unwrap();

        let request_result = handler(request, db.clone()).await.unwrap();
        let db_value = db.clone().get("1".to_string()).await.unwrap();

        assert_eq!(request_result.status(), StatusCode::OK);
        assert_eq!(db_value, Some("message".to_string()));
    }

    fn get_db() -> Arc<SledKeyValue> {
        let temp_dir = TempDir::new().expect("unable to create temp directory");
        Arc::new(SledKeyValue::new(temp_dir.path()).unwrap())
    }
}
