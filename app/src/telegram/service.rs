use async_trait::async_trait;
use reqwest::Client;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Telegram {
    async fn send_message(&self, chat_id: i32, message: &str) -> ();
}

pub struct TelegramService {
    client: Client,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct SendMessage {
    chat_id: i32,
    text: String,
}

impl TelegramService {
    pub fn new(api_key: &str) -> Self {
        TelegramService {
            client: Client::new(),
            path: format!("https://api.telegram.org/bot{}", api_key),
        }
    }
}

#[async_trait]
impl Telegram for TelegramService {
    async fn send_message(&self, chat_id: i32, message: &str) -> () {
        info!("sending message");
        let message = SendMessage {
            chat_id: chat_id,
            text: message.to_string(),
        };
        let resp = self
            .client
            .post(format!("{}/sendMessage", self.path))
            .json(&message)
            .send()
            .await
            .unwrap();
        let text = resp.text().await.unwrap();
        info!("response: {}", text);
    }
}
