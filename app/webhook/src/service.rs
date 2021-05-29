use std::sync::Arc;

use hyper::{
    body::{aggregate, Buf},
    Body, Method, Request, Response, StatusCode,
};
use telegram::{Telegram, types::Update};
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
    _telegram: Arc<RwLock<impl Telegram>>,
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
        Ok(_) => {
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
    use std::sync::Arc;
    use mockall::*;
    use async_trait::async_trait;

    use hyper::{body::to_bytes, Request, StatusCode};
    use telegram::Telegram;
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
}
