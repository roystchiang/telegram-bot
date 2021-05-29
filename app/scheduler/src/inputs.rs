use telegram::types::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreUpdate {
    pub update: Update,
}