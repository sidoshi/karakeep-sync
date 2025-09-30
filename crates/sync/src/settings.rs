use config::Config;
use serde::Deserialize;
use std::sync::OnceLock;

use crate::settings;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HNSettings {
    pub auth: Option<String>,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct KarakeepSettings {
    pub auth: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RedditSettings {
    pub clientid: Option<String>,
    pub clientsecret: Option<String>,
    pub refreshtoken: Option<String>,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Settings {
    pub hn: HNSettings,
    pub karakeep: KarakeepSettings,
    pub reddit: RedditSettings,
}

impl Settings {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();

        let config = Config::builder()
            .add_source(config::Environment::with_prefix("KS").separator("_"))
            .set_override("hn.schedule", "@daily")
            .unwrap()
            .set_override("reddit.schedule", "@daily")
            .unwrap()
            .build()
            .unwrap();

        let settings = config
            .try_deserialize::<settings::Settings>()
            .expect("Failed to deserialize settings");

        settings
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();
pub fn get_settings() -> &'static Settings {
    SETTINGS.get_or_init(Settings::new)
}
