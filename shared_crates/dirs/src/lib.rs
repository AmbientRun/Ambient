use std::{path::PathBuf, sync::OnceLock};

use directories::ProjectDirs;

/// Returns the path to the settings file.
pub fn settings_path() -> PathBuf {
    project_dirs().config_dir().to_owned().join("settings.toml")
}

/// Returns the path to the cache for the given deployment.
pub fn deployment_cache_path(deployment: &str) -> PathBuf {
    project_dirs()
        .cache_dir()
        .join("deployments")
        .join(deployment)
}

fn project_dirs() -> &'static ProjectDirs {
    const QUALIFIER: &str = "com";
    const ORGANIZATION: &str = "Ambient";
    const APPLICATION: &str = "Ambient";

    static CELL: OnceLock<ProjectDirs> = OnceLock::new();
    CELL.get_or_init(|| {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .expect("failed to open project directory for Ambient")
    })
}
