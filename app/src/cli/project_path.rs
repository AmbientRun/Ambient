use ambient_native_std::asset_url::AbsAssetUrl;
use anyhow::Context;
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone)]
pub struct ProjectPath {
    pub url: AbsAssetUrl,
    pub fs_path: Option<std::path::PathBuf>,
}

impl ProjectPath {
    pub fn new_local(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let current_dir = std::env::current_dir().context("Error getting current directory")?;
        let path = if path.is_absolute() {
            path
        } else {
            ambient_std::path::normalize(&current_dir.join(path))
        };

        if path.exists() && !path.is_dir() {
            anyhow::bail!("Project path {path:?} exists and is not a directory.");
        }
        let url = AbsAssetUrl::from_directory_path(path);
        let fs_path = url.to_file_path().ok().flatten();

        Ok(Self { url, fs_path })
    }

    pub fn is_remote(&self) -> bool {
        self.fs_path.is_none()
    }

    // 'static to limit only to compile-time known paths
    pub fn push(&self, path: &'static str) -> AbsAssetUrl {
        self.url.push(path).unwrap()
    }
}

impl TryFrom<Option<String>> for ProjectPath {
    type Error = anyhow::Error;

    fn try_from(project_path: Option<String>) -> anyhow::Result<Self> {
        match project_path {
            Some(project_path)
                if project_path.starts_with("http://")
                    || project_path.starts_with("https://")
                    || project_path.starts_with("file:/") =>
            {
                let url = AbsAssetUrl::from_str(&project_path)?;
                if let Some(local) = url.to_file_path()? {
                    Self::new_local(local)
                } else {
                    Ok(Self { url, fs_path: None })
                }
            }
            Some(project_path) => Self::new_local(project_path),
            None => {
                let url = AbsAssetUrl::from_directory_path(std::env::current_dir()?);
                let fs_path = url.to_file_path().ok().flatten();
                Ok(Self { url, fs_path })
            }
        }
    }
}
