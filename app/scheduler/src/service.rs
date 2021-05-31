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
                serde_json::to_string(&data.update)?,
            )
            .await?;

        println!("{:?}", data.update.update_id.to_string());
        Ok(Response::new("done".into()))
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use common::Server;
    use hyper::Body;
    use hyper::{Request, StatusCode};
    use scheduler::inputs::SchedulerUpdate;
    use storage::sled::SledKeyValue;
    use storage::KeyValue;
    use telegram::types::{Chat, Message, Update};
    use tempfile::TempDir;

    use crate::service::SchedulerService;

    #[tokio::test]
    async fn should_create_user_specific_store() {
        let temp_dir = TempDir::new()
            .expect("unable to create temp directory")
            .into_path();
        let server = SchedulerService::<SledKeyValue>::new(temp_dir.clone());
        let request = generate_request(1, 3);

        let request_result = server.serve(request).await.unwrap();
        let storage_keys = server.list_all_users().await;

        assert_eq!(request_result.status(), StatusCode::OK);
        assert_eq!(storage_keys, ["3"]);
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[tokio::test]
    async fn should_be_able_to_deserialize_message() {
        let temp_dir = TempDir::new()
            .expect("unable to create temp directory")
            .into_path();
        let server = SchedulerService::<SledKeyValue>::new(temp_dir.clone());
        let request = generate_request(1, 3);

        server.serve(request).await.unwrap();
        let stored_value = server
            .get_user_storage("3")
            .await
            .unwrap()
            .get("1".to_string())
            .await
            .unwrap()
            .unwrap();

        let _: Update = serde_json::from_str(&stored_value).unwrap();
        fs::remove_dir_all(temp_dir).unwrap();
    }

    fn generate_request(update_id: i32, chat_id: i32) -> Request<Body> {
        let input = SchedulerUpdate {
            update: Update {
                update_id,
                message: Message {
                    message_id: 2,
                    chat: Chat { id: chat_id },
                    text: Some("message".to_string()),
                    entities: None,
                },
            },
        };
        Request::builder()
            .body(serde_json::to_string(&input).unwrap().into())
            .unwrap()
    }
}
