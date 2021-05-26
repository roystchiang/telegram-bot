use reqwest::Client;

pub struct Telegram {
    client: Client,
    path: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub update_id: i32,
    pub message: Message
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i32,
    pub chat: Chat,
    pub text: Option<String>,
    pub entities: Option<Vec<MessageEntity>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageEntity{
    #[serde(rename = "type")]
    pub message_type: String,
    pub offset: i32,
    pub length: i32,
}

#[derive(Serialize, Deserialize)]
struct SendMessage {
    chat_id: i32,
    text: String
}


impl Telegram {
    pub fn new(api_key: &str) -> Self {
        Telegram {
            client: Client::new(),
            path: format!("https://api.telegram.org/bot{}", api_key),
        }
    }

    pub async fn send_message(&self, chat_id: i32, message: &str) {
        info!("sending message");
        let message = SendMessage {
            chat_id: chat_id,
            text: message.to_string(),
        };
        let resp = self.client
            .post(format!("{}/sendMessage", self.path))
            .json(&message)
            .send()
            .await.unwrap();
        let text = resp.text().await.unwrap();
        info!("response: {}", text);
    }
}