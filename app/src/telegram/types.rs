#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub update_id: i32,
    pub message: Message,
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
pub struct MessageEntity {
    #[serde(rename = "type")]
    pub message_type: String,
    pub offset: i32,
    pub length: i32,
}
