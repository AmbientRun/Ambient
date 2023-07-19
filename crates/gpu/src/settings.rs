use ambient_std::asset_cache::SyncAssetKey;
use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Settings {
    #[serde(default)]
    resolution: Resolution,
    #[serde(default)]
    vsync: Vsync,
    #[serde(default)]
    pub render_mode: RenderMode,
    #[serde(default)]
    pub software_culling: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderMode {
    #[default]
    MultiIndirect,
    Indirect,
    Direct,
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
    #[cfg(not(target_os = "unknown"))]
    pub fn load_from_file() -> Result<Settings> {
        use std::io::ErrorKind;

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
        let settings = std::fs::read_to_string(&settings_path);
        match settings {
            Ok(settings) => {
                let settings: Settings = toml::from_str(&settings)
                    .with_context(|| format!("Deserializing {FILE_NAME}"))?;
                Ok(settings)
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let settings = Settings::default();
                std::fs::write(&settings_path, toml::to_string(&settings)?)
                    .with_context(|| format!("Writing {FILE_NAME}"))?;
                Ok(settings)
            }
            Err(e) => Err(e).context("Error reading settings file at {settings_path:?}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsKey;

impl SyncAssetKey<Settings> for SettingsKey {
    fn load(&self, _assets: ambient_std::asset_cache::AssetCache) -> Settings {
        // #[cfg(target_os = "unknown")]
        {
            Settings {
                resolution: Resolution((800, 600)),
                vsync: Vsync(true),
                render_mode: RenderMode::Indirect,
                software_culling: false,
            }
        }
        // #[cfg(not(target_os = "unknown"))]
        // {
        //     match Settings::load_from_file() {
        //         Ok(settings) => settings,
        //         Err(error) => {
        //             tracing::warn!(
        //                 "Failed to load settings with error {error}. Fallback to defaults."
        //             );
        //             Settings::default()
        //         }
        //     }
        // }
    }
}
