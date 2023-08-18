use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use ambient_package::{BuildMetadata, Manifest, SnakeCaseIdentifier, Version};
use ambient_std::path;
use anyhow::Context;
use url::Url;

use crate::{util::read_file, Item, ItemData, ItemId, ItemType, ItemValue, Schema, Scope};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct PackageLocator {
    pub id: SnakeCaseIdentifier,
    pub version: Version,
    pub source: RetrievableFile,
}
impl PackageLocator {
    pub fn from_manifest(manifest: &Manifest, source: RetrievableFile) -> Self {
        Self {
            id: manifest.package.id.clone(),
            version: manifest.package.version.clone(),
            source,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
/// Paths should be to the manifest, not to the folder it's in
pub enum RetrievableFile {
    Ambient(PathBuf),
    Path(PathBuf),
    Url(Url),
}
impl RetrievableFile {
    pub async fn get(&self, schema: &Schema) -> anyhow::Result<String> {
        Ok(match self {
            RetrievableFile::Ambient(path) => schema
                .get(path.to_string_lossy().as_ref())
                .with_context(|| {
                    format!(
                        "failed to find path in Ambient schema: {}",
                        path.to_string_lossy()
                    )
                })?
                .to_string(),
            RetrievableFile::Path(path) => {
                anyhow::ensure!(path.is_absolute(), "path {path:?} must be absolute");
                let url = Url::from_file_path(path).unwrap();
                read_file(&url).await?
            }
            RetrievableFile::Url(url) => read_file(url).await?,
        })
    }

    /// Takes the parent of this path and joins it with the given path
    pub fn parent_join(&self, suffix: &Path) -> anyhow::Result<Self> {
        fn parent_join(path: &PathBuf, suffix: &Path) -> Result<PathBuf, anyhow::Error> {
            Ok(path::normalize(
                &path.parent().context("no parent")?.join(suffix),
            ))
        }

        Ok(match self {
            RetrievableFile::Ambient(path) => RetrievableFile::Ambient(parent_join(path, suffix)?),
            RetrievableFile::Path(path) => RetrievableFile::Path(parent_join(path, suffix)?),
            RetrievableFile::Url(url) => {
                RetrievableFile::Url(url.join(suffix.to_string_lossy().as_ref())?)
            }
        })
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct Package {
    pub data: ItemData,
    pub source: RetrievableFile,
    pub manifest: Manifest,
    pub build_metadata: Option<BuildMetadata>,
    pub dependencies: HashMap<SnakeCaseIdentifier, Dependency>,
    pub scope_id: ItemId<Scope>,
}
impl Item for Package {
    const TYPE: ItemType = ItemType::Package;

    type Unresolved = ();

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Package(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Package(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Package(self)
    }

    fn data(&self) -> &ItemData {
        &self.data
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Dependency {
    pub id: ItemId<Package>,
    /// On by default
    pub enabled: bool,
}
