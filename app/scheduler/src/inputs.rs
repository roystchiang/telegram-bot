use telegram::types::{Update};

#[derive(Debug, Serialize, Deserialize)]
pub struct SchedulerUpdate {
    pub update: Update,
}