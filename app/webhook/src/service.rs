use std::sync::Arc;

use hyper::{
    body::{aggregate, Buf},
    Body, Method, Request, Response, StatusCode,
};
use telegram::{types::Update, Telegram};
use tokio::sync::RwLock;

pub async fn handler(
    req: Request<Body>,
    telegram: Arc<RwLock<impl Telegram>>,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Ok(Response::new("healthy".into())),
        _ => handle_webhook(req, telegram).await,
    }
}

async fn handle_webhook(
    req: Request<Body>,
    telegram: Arc<RwLock<impl Telegram>>,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let body = match aggregate(req).await {
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
            let client = telegram.read().await;
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

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use mockall::*;
    use scheduler::inputs::SchedulerUpdate;
    use std::sync::Arc;

    use hyper::{body::to_bytes, Request, StatusCode};
    use telegram::{
        types::{Chat, Message, Update},
        Telegram,
    };
    use tokio::sync::RwLock;

    use super::handler;

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
        let request = Request::builder().body("".into()).unwrap();

        let result = handler(request, mock_telegram).await.unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(to_bytes(result).await.unwrap(), "Failed to parse input");
    }

    #[tokio::test]
    async fn should_return_200_for_health() {
        let mock_telegram = Arc::new(RwLock::new(MockTelegram::new()));
        let request = Request::builder().uri("/health").body("".into()).unwrap();

        let result = handler(request, mock_telegram).await.unwrap();

        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(to_bytes(result).await.unwrap(), "healthy");
    }

    #[tokio::test]
    async fn should_return_400_if_given_invalid_input() {
        let mock_telegram = Arc::new(RwLock::new(MockTelegram::new()));
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

        let result = handler(request, mock_telegram).await.unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
    }
}
