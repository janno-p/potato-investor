use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub discord_token: String,
    pub eventstore_address: String,
    pub bot_user_id: u64,
    pub self_user_id: u64,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        config
            .merge(File::with_name("settings"))?
            .merge(File::with_name("settings.user").required(false))?
            .merge(Environment::with_prefix("RUST_LOG"))?;

        config.try_into()
    }
}
