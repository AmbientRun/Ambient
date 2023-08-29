use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use ambient_package::{BuildMetadata, Manifest, SnakeCaseIdentifier, Version};
use ambient_std::path;
use anyhow::Context as AnyhowContext;
use url::Url;

use crate::{
    item::ResolveClone, schema, util::read_file, Context, Item, ItemData, ItemId, ItemMap,
    ItemType, ItemValue, Scope, StandardDefinitions,
};

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
impl Display for PackageLocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}[", self.id, self.version)?;
        match &self.source {
            RetrievableFile::Ambient(p) => write!(f, "ambient:{}", p.display()),
            RetrievableFile::Path(p) => write!(f, "path:{}", p.display()),
            RetrievableFile::Url(u) => write!(f, "url:{}", u),
        }?;
        write!(f, "]")
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
    pub async fn get(&self) -> anyhow::Result<String> {
        Ok(match self {
            RetrievableFile::Ambient(path) => schema()
                .get(ambient_std::path::path_to_unix_string_lossy(path).as_str())
                .with_context(|| {
                    format!(
                        "failed to find path in Ambient schema: {}",
                        path.to_string_lossy()
                    )
                })?
                .to_string(),
            RetrievableFile::Path(path) => {
                anyhow::ensure!(path.is_absolute(), "path {path:?} must be absolute");
                #[cfg(target_os = "unknown")]
                {
                    unimplemented!("file reading is not supported on web")
                }

                #[cfg(not(target_os = "unknown"))]
                {
                    let url = Url::from_file_path(path).unwrap();
                    read_file(&url).await?
                }
            }
            RetrievableFile::Url(url) => read_file(url).await?,
        })
    }

    /// Takes the parent of this path and joins it with the given path
    pub fn parent_join(&self, suffix: &Path) -> anyhow::Result<Self> {
        fn parent_join(path: &Path, suffix: &Path) -> anyhow::Result<PathBuf> {
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

    pub fn as_local_or_remote(&self) -> Option<LocalOrRemote> {
        match self {
            Self::Ambient(_) => None,
            Self::Path(v) => Some(LocalOrRemote::Local(v.to_owned())),
            Self::Url(url) => {
                #[cfg(target_os = "unknown")]
                {
                    let _url = url;
                    unimplemented!("getting the path of an url is not supported on web")
                }

                #[cfg(not(target_os = "unknown"))]
                {
                    if url.scheme() == "file" {
                        Some(LocalOrRemote::Local(url.to_file_path().ok()?))
                    } else {
                        Some(LocalOrRemote::Remote(url.clone()))
                    }
                }
            }
        }
    }

    pub fn as_local_path(&self) -> Option<PathBuf> {
        match self.as_local_or_remote()? {
            LocalOrRemote::Local(v) => Some(v),
            LocalOrRemote::Remote(_) => None,
        }
    }

    pub fn as_remote_url(&self) -> Option<Url> {
        match self.as_local_or_remote()? {
            LocalOrRemote::Local(_) => None,
            LocalOrRemote::Remote(v) => Some(v.clone()),
        }
    }
}
impl Display for RetrievableFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetrievableFile::Ambient(path) => write!(f, "ambient://{}", path.display()),
            RetrievableFile::Path(path) => write!(f, "file://{}", path.display()),
            RetrievableFile::Url(url) => write!(f, "{}", url),
        }
    }
}

pub enum LocalOrRemote {
    Local(PathBuf),
    Remote(Url),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Package {
    pub data: ItemData,
    pub locator: PackageLocator,
    pub source: RetrievableFile,
    pub manifest: Manifest,
    pub build_metadata: Option<BuildMetadata>,
    pub dependencies: HashMap<SnakeCaseIdentifier, Dependency>,
    pub scope_id: ItemId<Scope>,
    /// The package that this package was imported from, if any
    pub dependent_package_id: Option<ItemId<Package>>,
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
impl ResolveClone for Package {
    fn resolve_clone(
        self,
        items: &mut ItemMap,
        context: &Context,
        definitions: &StandardDefinitions,
        _self_id: ItemId<Self>,
    ) -> anyhow::Result<Self> {
        items.resolve_clone(context, definitions, self.scope_id)?;
        Ok(self)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Dependency {
    pub id: ItemId<Package>,
    pub enabled: Option<bool>,
}
