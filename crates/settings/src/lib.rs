use ambient_native_std::asset_cache::SyncAssetKey;
use serde::{Deserialize, Serialize};

#[cfg(not(target_os = "unknown"))]
use std::path::PathBuf;

#[cfg(not(target_os = "unknown"))]
use anyhow::Context;

#[cfg(not(target_os = "unknown"))]
use directories::ProjectDirs;

mod render;
pub use render::*;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Settings {
    #[serde(default)]
    pub general: GeneralSettings,
    pub render: RenderSettings,
}
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct GeneralSettings {
    pub api_token: Option<String>,
}
#[cfg(not(target_os = "unknown"))]
impl Settings {
    pub fn load_from_file() -> anyhow::Result<Settings> {
        use std::io::ErrorKind;

        let path = Self::path()?;
        let settings = std::fs::read_to_string(&path);
        match settings {
            Ok(settings) => {
                Ok(match toml::from_str(&settings) {
                    Ok(settings) => settings,
                    Err(err) => {
                        if let Ok(render) = toml::from_str::<RenderSettings>(&settings) {
                            // TEMP: Migrate old settings, which only had render settings,
                            // to the new format
                            let settings = Settings {
                                render,
                                ..Default::default()
                            };
                            settings.write_to_file(Some(path.clone()))?;
                            settings
                        } else {
                            return Err(err)
                                .with_context(|| format!("Failed to parse settings at {path:?}"));
                        }
                    }
                })
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let settings = Settings::default();
                settings.write_to_file(Some(path.clone()))?;
                Ok(settings)
            }
            Err(e) => Err(e).with_context(|| format!("Error reading settings file at {path:?}")),
        }
    }

    pub fn write_to_file(&self, path: Option<PathBuf>) -> anyhow::Result<()> {
        let path = match path {
            Some(path) => path,
            None => Self::path()?,
        };
        Ok(std::fs::write(&path, toml::to_string(self)?)
            .with_context(|| format!("Failed to write settings to {path:?}"))?)
    }

    pub fn path() -> anyhow::Result<PathBuf> {
        const QUALIFIER: &str = "com";
        const ORGANIZATION: &str = "Ambient";
        const APPLICATION: &str = "Ambient";
        const FILE_NAME: &str = "settings.toml";

        let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .context("Failed to open home directory")?;

        let settings_dir = project_dirs.config_dir();
        if !settings_dir.exists() {
            std::fs::create_dir_all(settings_dir).with_context(|| {
                format!(
                    "Failed to create {APPLICATION} settings directory at {}",
                    settings_dir.display()
                )
            })?;
        }

        Ok(settings_dir.join(FILE_NAME))
    }
}

#[derive(Debug, Clone)]
pub struct SettingsKey;

impl SyncAssetKey<Settings> for SettingsKey {
    fn load(&self, _assets: ambient_native_std::asset_cache::AssetCache) -> Settings {
        #[cfg(target_os = "unknown")]
        {
            Settings {
                render: RenderSettings {
                    render_mode: RenderMode::Indirect,
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        #[cfg(not(target_os = "unknown"))]
        {
            match Settings::load_from_file() {
                Ok(settings) => settings,
                Err(error) => {
                    tracing::warn!(
                        "Failed to load settings with error {error}. Fallback to defaults."
                    );
                    Settings::default()
                }
            }
        }
    }
}
