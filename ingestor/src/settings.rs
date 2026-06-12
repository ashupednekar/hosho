use config::{Config, ConfigError, Environment};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings {}

impl Settings {
    fn new() -> Result<Self, ConfigError> {
        let conf = Config::builder()
            .add_source(Environment::default())
            .build()?;
        conf.try_deserialize()
    }
}

lazy_static! {
    pub static ref settings: Settings = Settings::new().expect("improperly configured");
}
