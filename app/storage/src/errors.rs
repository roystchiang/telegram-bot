use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyValueError {
    // Represents a failure coming from Sled
    #[error("Opeartional error")]
    OperationError { source: sled::Error },

    // Represents a failure when deserializing value
    #[error("Deserialize error")]
    DeserialzeError { source: FromUtf8Error },
}
