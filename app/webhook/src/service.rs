use std::sync::Arc;

use async_trait::async_trait;
use common::Server;
use hyper::{
    body::{aggregate, Buf},
    Body, Request, Response, StatusCode,
};
use telegram::{types::Update, Telegram};
use tokio::sync::RwLock;

pub struct WebhookSerice<T>
where
    T: Telegram,
{
    telegram: Arc<RwLock<T>>,
}

impl<T: Telegram> WebhookSerice<T> {
    pub fn new(telegram: Arc<RwLock<T>>) -> Self {
        Self { telegram }
    }
}

#[async_trait]
impl<T: Telegram + Send + Sync> Server for WebhookSerice<T> {
    async fn serve(
        &self,
        request: Request<Body>,
    ) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
        let body = match aggregate(request).await {
            Ok(body) => body,
            Err(e) => {
                error!("Failed to load request body: {}", e);
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Internal error".into())
                    .unwrap();
                return Ok(response);
            }
        };

        match serde_json::from_reader::<_, Update>(body.reader()) {
            Ok(update) => {
                let client = self.telegram.read().await;
                client.send_message(update.message.chat.id, "ack").await;
                return Ok(Response::new("Ok".into()));
            }
            Err(e) => {
                warn!("Failed to deserialize JSON: {}", e);
                let response = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("Failed to parse input".into())
                    .unwrap();
                return Ok(response);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use mockall::*;
    use scheduler::inputs::SchedulerUpdate;
    use std::sync::Arc;

    use common::Server;
    use hyper::{body::to_bytes, Request, StatusCode};
    use telegram::{
        types::{Chat, Message, Update},
        Telegram,
    };
    use tokio::sync::RwLock;

    use crate::service::WebhookSerice;

    mock! {
        Telegram {}

        #[async_trait]
        impl Telegram for Telegram {
            async fn send_message(&self, chat_id: i32, message: &str) -> ();
        }
    }

    #[tokio::test]
    async fn should_return_400_if_no_input() {
        let mock_telegram = Arc::new(RwLock::new(MockTelegram::new()));
        let server = WebhookSerice::new(mock_telegram);
        let request = Request::builder().body("".into()).unwrap();

        let result = server.serve(request).await.unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(to_bytes(result).await.unwrap(), "Failed to parse input");
    }

    #[tokio::test]
    async fn should_return_400_if_given_invalid_input() {
        let mock_telegram = Arc::new(RwLock::new(MockTelegram::new()));
        let server = WebhookSerice::new(mock_telegram);
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
            .uri("/")
            .body(serde_json::to_string(&input).unwrap().into())
            .unwrap();

        let result = server.serve(request).await.unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
    }
}
