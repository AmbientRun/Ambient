use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct GeneralSettings {
    pub api_token: Option<String>,
    pub sentry: Sentry,
}

impl GeneralSettings {
    pub fn is_sentry_enabled(&self) -> bool {
        self.sentry.0 .0
    }

    pub fn sentry_dsn(&self) -> String {
        self.sentry.0 .1.clone()
    }
}

// enabled, dsn
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sentry(pub (bool, String));
impl Default for Sentry {
    fn default() -> Self {
        Self((
            true,
            "https://e9aa91729019feba879bce792f32b119@o943373.ingest.sentry.io/4505715308101632"
                .to_string(),
        ))
    }
}
