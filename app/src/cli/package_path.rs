use ambient_native_std::asset_url::AbsAssetUrl;
use anyhow::Context;
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone)]
pub struct PackagePath {
    pub url: AbsAssetUrl,
    pub fs_path: Option<std::path::PathBuf>,
}

impl PackagePath {
    pub fn new_local(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let current_dir = std::env::current_dir().context("Error getting current directory")?;
        let path = if path.is_absolute() {
            path
        } else {
            ambient_std::path::normalize(&current_dir.join(path))
        };

        if path.exists() && !path.is_dir() {
            anyhow::bail!("Package path {path:?} exists and is not a directory.");
        }
        let url = AbsAssetUrl::from_directory_path(path);
        let fs_path = url.to_file_path().ok().flatten();

        Ok(Self { url, fs_path })
    }

    pub fn is_remote(&self) -> bool {
        self.fs_path.is_none()
    }
}

impl TryFrom<Option<String>> for PackagePath {
    type Error = anyhow::Error;

    fn try_from(package_path: Option<String>) -> anyhow::Result<Self> {
        match package_path {
            Some(package_path)
                if package_path.starts_with("http://")
                    || package_path.starts_with("https://")
                    || package_path.starts_with("file:/") =>
            {
                let url = AbsAssetUrl::from_str(&package_path)?;
                if url.extension().is_some() {
                    anyhow::bail!("Package path must be a directory");
                }

                let url = url.as_directory();
                if let Some(local) = url.to_file_path()? {
                    Self::new_local(local)
                } else {
                    Ok(Self { url, fs_path: None })
                }
            }
            Some(package_path) => Self::new_local(package_path),
            None => {
                let url = AbsAssetUrl::from_directory_path(std::env::current_dir()?);
                let fs_path = url.to_file_path().ok().flatten();
                Ok(Self { url, fs_path })
            }
        }
    }
}
