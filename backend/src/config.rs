use ::config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

// The App config
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub run_migrations: bool,
    pub server_addr: String,
    pub allow_debug_headers: bool,
    pub gpg_email: Option<String>,
}

// Load up the config
impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config"))
            .add_source(Environment::default())
            .build()?;

        s.try_deserialize()
    }
}
