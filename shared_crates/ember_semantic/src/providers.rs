use std::path::{Path, PathBuf};

use ambient_std::path;
use async_trait::async_trait;

#[async_trait]
pub trait FileProvider: Sync + Send {
    async fn get(&self, path: &Path) -> std::io::Result<String>;
    fn full_path(&self, path: &Path) -> PathBuf;
}

/// Implements [FileProvider] by reading from the filesystem.
pub struct DiskFileProvider(pub PathBuf);
#[async_trait]
impl FileProvider for DiskFileProvider {
    async fn get(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(self.0.join(path))
    }

    fn full_path(&self, path: &Path) -> PathBuf {
        path::normalize(&self.0.join(path))
    }
}

/// Implements [FileProvider] by reading from an array of files.
///
/// Used with `ambient_schema`.
pub struct ArrayFileProvider<'a> {
    pub files: &'a [(&'a str, &'a str)],
}
impl ArrayFileProvider<'_> {
    pub fn from_schema() -> Self {
        Self {
            files: ambient_schema::FILES,
        }
    }
}
#[async_trait]
impl FileProvider for ArrayFileProvider<'_> {
    async fn get(&self, path: &Path) -> std::io::Result<String> {
        let path = path.to_str().unwrap();
        for (name, contents) in self.files {
            if path == *name {
                return Ok(contents.to_string());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("file not found: {:?}", path),
        ))
    }

    fn full_path(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }
}

pub struct ProxyFileProvider<'a> {
    pub provider: &'a dyn FileProvider,
    pub base: &'a Path,
}
#[async_trait]
impl FileProvider for ProxyFileProvider<'_> {
    async fn get(&self, path: &Path) -> std::io::Result<String> {
        self.provider.get(&self.base.join(path)).await
    }

    fn full_path(&self, path: &Path) -> PathBuf {
        path::normalize(&self.provider.full_path(&self.base.join(path)))
    }
}
