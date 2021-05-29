use std::sync::Arc;

use hyper::{
    body::{aggregate, Buf},
    Body, Request, Response, StatusCode,
};
use tokio::sync::RwLock;

use crate::telegram::{service::Telegram, types::Update};

pub async fn handler(
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

    use crate::telegram::service::MockTelegram;
    use hyper::{
        body::{to_bytes},
        Request, StatusCode,
    };
    use tokio::sync::RwLock;

    use super::handler;

    #[tokio::test]
    async fn should_return_400_if_no_input() {
        let mock_telegram = Arc::new(RwLock::new(MockTelegram::new()));
        let request = Request::builder().body("".into()).unwrap();

        let result = handler(request, mock_telegram).await.unwrap();

        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        assert_eq!(to_bytes(result).await.unwrap(), "Failed to parse input");
    }
}
