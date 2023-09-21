use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct GeneralSettings {
    pub user_id: Option<String>,
    pub api_token: Option<String>,
    pub sentry: Sentry,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sentry {
    pub enabled: bool,
    pub dsn: String,
}

impl Default for Sentry {
    fn default() -> Self {
        Self {
            enabled: true,
            dsn:
                "https://e9aa91729019feba879bce792f32b119@o943373.ingest.sentry.io/4505715308101632"
                    .to_string(),
        }
    }
}
