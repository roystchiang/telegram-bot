use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
pub struct TelegramConfig {
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub telegram: TelegramConfig,
}

impl Settings {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(path))?;
        s.try_into()
    }
}