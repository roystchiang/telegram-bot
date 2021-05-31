pub mod errors;
pub mod sled;

use std::path::PathBuf;

use anyhow::Result;

use async_trait::async_trait;
use errors::KeyValueError;

#[async_trait]
pub trait KeyValue {
    fn new(path: &PathBuf) -> Result<Self>
    where
        Self: Sized;

    async fn get(&self, key: String) -> Result<Option<String>, KeyValueError>;

    async fn set(&self, key: String, value: String) -> Result<(), KeyValueError>;
}
