use std::sync::Arc;

use async_trait::async_trait;
use hyper::body::Buf;
use hyper::{body::aggregate, Body, Request, Response};
use scheduler::inputs::SchedulerUpdate;
use storage::KeyValue;

use common::Server;

pub struct SchedulerService<T>
where
    T: KeyValue,
{
    storage: Arc<T>,
}

impl<T: KeyValue> SchedulerService<T> {
    pub fn new(storage: Arc<T>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<T: KeyValue + Send + Sync> Server for SchedulerService<T> {
    async fn serve(
        &self,
        request: Request<Body>,
    ) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Serving request on uri: {}", request.uri());
        let body = aggregate(request).await?;
        let data = serde_json::from_reader::<_, SchedulerUpdate>(body.reader())?;

        self.storage
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
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use common::Server;
    use hyper::{Request, StatusCode};
    use scheduler::inputs::SchedulerUpdate;
    use storage::sled::SledKeyValue;
    use storage::KeyValue;
    use telegram::types::{Chat, Message, Update};
    use tempfile::TempDir;

    use crate::service::SchedulerService;

    #[tokio::test]
    async fn should_store_received_message_update() {
        let db = get_db();
        let server = SchedulerService::new(db.clone());
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

        let request_result = server.serve(request).await.unwrap();
        let db_value = db.clone().get("1".to_string()).await.unwrap();

        assert_eq!(request_result.status(), StatusCode::OK);
        assert_eq!(db_value, Some("message".to_string()));
    }

    fn get_db() -> Arc<SledKeyValue> {
        let temp_dir = TempDir::new().expect("unable to create temp directory");
        Arc::new(SledKeyValue::new(temp_dir.path()).unwrap())
    }
}
