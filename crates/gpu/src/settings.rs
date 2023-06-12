use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Settings {
    #[serde(default)]
    resolution: Resolution,
    #[serde(default)]
    vsync: Vsync,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Resolution((u32, u32));

impl Default for Resolution {
    fn default() -> Self {
        Self((800, 600))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Vsync(bool);

impl Default for Vsync {
    fn default() -> Self {
        Self(true)
    }
}

impl Settings {
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution.0
    }

    pub fn vsync(&self) -> bool {
        self.vsync.0
    }
}

impl Settings {
    pub fn load_from_config() -> Result<Settings> {
        const QUALIFIER: &str = "com";
        const ORGANIZATION: &str = "Ambient";
        const APPLICATION: &str = "Ambient";
        const FILE_NAME: &str = "settings.toml";

        let Some(project_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) else {
            bail!("Failed to open home directory");
        };

        let settings_dir = project_dirs.config_dir();
        if !settings_dir.exists() {
            std::fs::create_dir_all(settings_dir).with_context(|| {
                format!(
                    "Creating {APPLICATION} settings directory at {}",
                    settings_dir.display()
                )
            })?;
        }

        let settings_path = settings_dir.join(FILE_NAME);
        tracing::info!("Reading {FILE_NAME} from {}", settings_path.display());
        let settings = if settings_path.exists() {
            let settings = std::fs::read_to_string(&settings_path)?;
            let settings: Settings =
                toml::from_str(&settings).with_context(|| format!("Deserializing {FILE_NAME}"))?;
            settings
        } else {
            Settings::default()
        };

        std::fs::write(&settings_path, toml::to_string(&settings)?)
            .with_context(|| format!("Writing {FILE_NAME}"))?;

        Ok(settings)
    }
}
