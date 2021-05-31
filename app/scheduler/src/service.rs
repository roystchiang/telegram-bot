use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use hyper::body::Buf;
use hyper::{body::aggregate, Body, Request, Response};
use scheduler::inputs::SchedulerUpdate;
use storage::KeyValue;

use anyhow::Result;
use common::Server;
use tokio::sync::RwLock;

pub struct SchedulerService<T>
where
    T: KeyValue,
{
    path: PathBuf,
    storage: Arc<RwLock<BTreeMap<String, Arc<T>>>>,
}

impl<T: KeyValue> SchedulerService<T> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            storage: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    async fn add_user_storage(&self, user_id: &str) -> Result<Arc<T>> {
        let mut storage = self.storage.write().await;
        // read again to make sure that another thread has not created the KV store before we got the lock
        match storage.get(user_id) {
            Some(kv) => Ok(kv.clone()),
            None => {
                let kv_store_path = self.get_kv_path(user_id);
                let kv_store: Arc<T> = Arc::new(T::new(&kv_store_path)?);
                storage.insert(user_id.to_string(), kv_store.clone());
                Ok(kv_store.clone())
            }
        }
    }

    async fn get_user_storage(&self, user_id: &str) -> Option<Arc<T>> {
        let storage_read = self.storage.read().await;
        match storage_read.get(user_id) {
            Some(kv) => Some(kv.clone()),
            None => None,
        }
    }

    async fn list_all_users(&self) -> Vec<String> {
        self.storage.read().await.keys().cloned().collect()
    }

    fn get_kv_path(&self, user_id: &str) -> PathBuf {
        let mut kv_store_path = (&self.path).clone();
        kv_store_path.push(user_id);
        println!("{:?}", kv_store_path);
        kv_store_path
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

        let storage_key = data.update.message.chat.id.to_string();
        let storage = self
            .get_user_storage(&storage_key)
            .await
            .unwrap_or(self.add_user_storage(&storage_key).await?);

        storage
            .clone()
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
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
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
        let temp_dir = TempDir::new()
            .expect("unable to create temp directory")
            .into_path();
        let server = SchedulerService::<SledKeyValue>::new(temp_dir.clone());
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
        let storage_keys = server.list_all_users().await;

        assert_eq!(request_result.status(), StatusCode::OK);
        assert_eq!(storage_keys, ["3"]);
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
