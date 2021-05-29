pub mod sled;
pub mod errors;

use anyhow::{Result};

use async_trait::async_trait;
use errors::KeyValueError;

#[async_trait]
pub trait KeyValue {
    async fn get(&self, key: String) -> Result<Option<String>, KeyValueError>;

    async fn set(&self, key: String, value: String) -> Result<(), KeyValueError>;
}
