use ambient_native_std::asset_cache::SyncAssetKey;
use serde::{Deserialize, Serialize};

#[cfg(not(target_os = "unknown"))]
use std::path::PathBuf;

#[cfg(not(target_os = "unknown"))]
use anyhow::Context;

mod render;
pub use render::*;

mod general;
pub use general::*;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Settings {
    #[serde(default)]
    pub general: GeneralSettings,
    pub render: RenderSettings,
}

#[cfg(not(target_os = "unknown"))]
impl Settings {
    pub fn load_from_file() -> anyhow::Result<Settings> {
        use std::io::ErrorKind;

        let path = Self::path()?;
        let settings = std::fs::read_to_string(&path);
        match settings {
            Ok(settings) => {
                Ok(match toml::from_str::<Settings>(&settings) {
                    Ok(settings) => {
                        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
                        if settings.render.render_mode == Some(RenderMode::MultiIndirect) {
                            tracing::warn!("MultiIndirect render mode is not supported on this platform. Falling back to Indirect");
                            let mut settings = settings;
                            settings.render.render_mode = Some(RenderMode::Indirect);

                            let _ = settings.write_to_file(Some(path.clone()));

                            settings
                        } else {
                            settings
                        }
                        #[cfg(any(target_os = "windows", target_os = "linux"))]
                        {
                            settings
                        }
                    }
                    Err(err) => {
                        if let Ok(render) = toml::from_str::<RenderSettings>(&settings) {
                            // TEMP: Migrate old settings, which only had render settings,
                            // to the new format
                            let settings = Settings {
                                render,
                                ..Default::default()
                            };

                            settings.write_to_file(Some(path))?;

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
                settings.write_to_file(Some(path))?;
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
        std::fs::write(&path, toml::to_string(self)?)
            .with_context(|| format!("Failed to write settings to {path:?}"))
    }

    pub fn path() -> anyhow::Result<PathBuf> {
        let path = ambient_dirs::settings_path();
        tracing::error!("Reading settings from: {:?}", path);

        if let Some(parent) = path.parent().filter(|p| !p.exists()) {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create Ambient settings directory at {}",
                    parent.display()
                )
            })?;
        }

        Ok(path)
    }
}

#[derive(Debug, Clone)]
pub struct SettingsKey;

impl SyncAssetKey<Settings> for SettingsKey {
    fn load(&self, _assets: ambient_native_std::asset_cache::AssetCache) -> Settings {
        #[cfg(target_os = "unknown")]
        {
            use js_sys::Reflect;
            let nav = web_sys::window().unwrap().navigator();
            let ua = Reflect::get(&nav, &"userAgentData".into()).unwrap();
            let platform = Reflect::get(&ua, &"platform".into()).unwrap().as_string();

            tracing::info!(?platform, "Detected platform");
            if platform.as_deref() == Some("Windows") {
                Settings {
                    render: RenderSettings {
                        render_mode: Some(RenderMode::Direct),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            } else {
                Settings {
                    render: RenderSettings {
                        render_mode: Some(RenderMode::Direct),
                        ..Default::default()
                    },
                    ..Default::default()
                }
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
