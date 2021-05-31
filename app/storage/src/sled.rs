use std::path::PathBuf;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sled::{open, Db};

use crate::{errors::KeyValueError, KeyValue};

pub struct SledKeyValue {
    db: Db,
}

#[async_trait]
impl KeyValue for SledKeyValue {
    fn new(path: &PathBuf) -> Result<Self> {
        let db = open(path).context(format!("unable to open db '{:?}'", path.to_str()))?;

        Ok(Self { db })
    }

    async fn get(&self, key: String) -> Result<Option<String>, KeyValueError> {
        let value = self.db.get(key).unwrap();
        value
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()
            .map_err(|source| KeyValueError::DeserialzeError { source })
    }

    async fn set(&self, key: String, value: String) -> Result<(), KeyValueError> {
        self.db
            .insert(key, value.into_bytes())
            .and_then(|_| self.db.flush())
            .map(|_| ())
            .map_err(|source| KeyValueError::OperationError { source })
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use tempfile::TempDir;

    use crate::KeyValue;

    use super::SledKeyValue;

    #[tokio::test]
    async fn should_return_key() {
        let temp_dir = TempDir::new()
            .expect("unable to create temp directory")
            .into_path();
        let db = SledKeyValue::new(&temp_dir).unwrap();

        db.set("test-key".to_string(), "some-value".to_string())
            .await
            .unwrap();

        assert_eq!(
            db.get("test-key".to_string()).await.unwrap().unwrap(),
            "some-value"
        );
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[tokio::test]
    async fn should_not_fail_if_key_does_not_exist() {
        let temp_dir = TempDir::new()
            .expect("unable to create temp directory")
            .into_path();
        let db = SledKeyValue::new(&temp_dir).unwrap();

        let value = db.get("random key".to_string()).await;

        assert_eq!(value.unwrap(), None);
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
