use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use ambient_package::{BuildMetadata, Manifest, SnakeCaseIdentifier, Version};
use anyhow::Context;
use url::Url;

use crate::{
    util::{ensure_url_is_directory, read_file},
    Item, ItemData, ItemId, ItemType, ItemValue, Schema, Scope,
};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct PackageLocator {
    pub id: SnakeCaseIdentifier,
    pub version: Version,
    pub source: PackageSource,
}
impl PackageLocator {
    pub fn from_manifest(manifest: &Manifest, source: PackageSource) -> Self {
        Self {
            id: manifest.package.id.clone(),
            version: manifest.package.version.clone(),
            source,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
/// Paths should be to the root of the package, not the manifest
pub enum PackageSource {
    Ambient(PathBuf),
    Path(PathBuf),
    Url(Url),
}
impl PackageSource {
    pub async fn get_manifest(&self, schema: &Schema) -> anyhow::Result<(Manifest, Option<Url>)> {
        let (manifest, url) = match self {
            PackageSource::Ambient(path) => (
                schema
                    .get(path.join("ambient.toml").to_string_lossy().as_ref())
                    .with_context(|| {
                        format!(
                            "failed to find ambient.toml for path: {}",
                            path.to_string_lossy()
                        )
                    })?
                    .to_string(),
                None,
            ),
            PackageSource::Path(path) => {
                anyhow::ensure!(path.is_absolute(), "path must be absolute");
                let url = Url::from_file_path(path.join("ambient.toml")).unwrap();
                (read_file(&url).await?, Some(url))
            }
            PackageSource::Url(url) => {
                let url = ensure_url_is_directory(url.clone()).join("ambient.toml")?;
                (read_file(&url).await?, Some(url))
            }
        };

        Ok((Manifest::parse(&manifest)?, url))
    }

    pub fn join(&self, suffix: &Path) -> anyhow::Result<Self> {
        anyhow::ensure!(
            suffix.extension().is_none(),
            "path {} must not have an extension (i.e. must not be a file)",
            suffix.display()
        );
        Ok(match self {
            PackageSource::Ambient(path) => PackageSource::Path(path.join(suffix)),
            PackageSource::Path(path) => PackageSource::Path(path.join(suffix)),
            PackageSource::Url(url) => PackageSource::Url(
                ensure_url_is_directory(url.clone()).join(&suffix.to_string_lossy())?,
            ),
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Package {
    pub data: ItemData,
    pub source: PackageSource,
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
