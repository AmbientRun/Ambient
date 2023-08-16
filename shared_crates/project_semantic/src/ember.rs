use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use ambient_project::{BuildMetadata, Manifest, SnakeCaseIdentifier, Version};
use anyhow::Context;
use url::Url;

use crate::{
    util::{ensure_url_is_directory, read_file},
    Item, ItemData, ItemId, ItemType, ItemValue, Schema, Scope,
};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct EmberLocator {
    pub id: SnakeCaseIdentifier,
    pub version: Version,
    pub source: EmberSource,
}
impl EmberLocator {
    pub fn from_manifest(manifest: &Manifest, source: EmberSource) -> Self {
        Self {
            id: manifest.ember.id.clone(),
            version: manifest.ember.version.clone(),
            source,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
/// Paths should be to the root of the ember, not the manifest
pub enum EmberSource {
    Ambient(PathBuf),
    Path(PathBuf),
    Url(Url),
}
impl EmberSource {
    pub async fn get_manifest(&self, schema: &Schema) -> anyhow::Result<(Manifest, Option<Url>)> {
        let (manifest, url) = match self {
            EmberSource::Ambient(path) => (
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
            EmberSource::Path(path) => {
                anyhow::ensure!(path.is_absolute(), "path must be absolute");
                let url = Url::from_file_path(path.join("ambient.toml")).unwrap();
                (read_file(&url).await?, Some(url))
            }
            EmberSource::Url(url) => {
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
            EmberSource::Ambient(path) => EmberSource::Path(path.join(suffix)),
            EmberSource::Path(path) => EmberSource::Path(path.join(suffix)),
            EmberSource::Url(url) => EmberSource::Url(
                ensure_url_is_directory(url.clone()).join(&suffix.to_string_lossy())?,
            ),
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Ember {
    pub data: ItemData,
    pub source: EmberSource,
    pub manifest: Manifest,
    pub build_metadata: Option<BuildMetadata>,
    pub dependencies: HashMap<SnakeCaseIdentifier, Dependency>,
    pub scope_id: ItemId<Scope>,
}
impl Item for Ember {
    const TYPE: ItemType = ItemType::Ember;

    type Unresolved = ();

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Ember(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Ember(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Ember(self)
    }

    fn data(&self) -> &ItemData {
        &self.data
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Dependency {
    pub id: ItemId<Ember>,
    /// On by default
    pub enabled: bool,
}
