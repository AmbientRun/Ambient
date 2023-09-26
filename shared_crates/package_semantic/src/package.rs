use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use ambient_package::{BuildMetadata, Manifest, PackageId, SnakeCaseIdentifier};
use ambient_std::path;
use thiserror::Error;
use url::Url;

use crate::{
    schema,
    util::{read_file, ReadFileError},
    Item, ItemData, ItemId, ItemType, ItemValue, Resolve, Scope, Semantic,
};
use semver::Version;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct PackageLocator {
    pub id: PackageId,
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

#[derive(Error, Debug)]
pub enum GetError {
    #[error("Failed to find {0:?} in Ambient schema")]
    FailedToFindInAmbientSchema(PathBuf),
    #[error("Failed to read file")]
    ReadFileError(#[from] ReadFileError),
    #[error("Path {0:?} must be absolute")]
    PathMustBeAbsolute(PathBuf),
}

#[derive(Error, Debug)]
pub enum ParentJoinError {
    #[error("No parent for {0:?}")]
    NoParent(PathBuf),
    #[error("URL parse error")]
    UrlParseError(#[from] url::ParseError),
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
/// Paths should be to the manifest, not to the folder it's in
pub enum RetrievableFile {
    Ambient(PathBuf),
    Path(PathBuf),
    Url(Url),
}
impl RetrievableFile {
    pub async fn get(&self) -> Result<String, GetError> {
        Ok(match self {
            RetrievableFile::Ambient(path) => schema()
                .get(ambient_std::path::path_to_unix_string_lossy(path).as_str())
                .ok_or_else(|| GetError::FailedToFindInAmbientSchema(path.to_owned()))?
                .to_string(),
            RetrievableFile::Path(path) => {
                if !path.is_absolute() {
                    return Err(GetError::PathMustBeAbsolute(path.to_owned()));
                }

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
    pub fn parent_join(&self, suffix: &Path) -> Result<Self, ParentJoinError> {
        fn parent_join(path: &Path, suffix: &Path) -> Result<PathBuf, ParentJoinError> {
            Ok(path::normalize(
                &path
                    .parent()
                    .ok_or_else(|| ParentJoinError::NoParent(path.to_owned()))?
                    .join(suffix),
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
            LocalOrRemote::Remote(v) => Some(v),
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

    pub(super) resolved: bool,
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
impl Resolve for Package {
    fn resolve(mut self, semantic: &mut Semantic, _self_id: ItemId<Self>) -> anyhow::Result<Self> {
        // Ensure all dependencies are resolved first, so that we can use them
        // when resolving ourselves
        for dependency in self.dependencies.values_mut() {
            semantic.resolve(dependency.id)?;
        }

        semantic.resolve(self.scope_id)?;
        self.resolved = true;
        Ok(self)
    }

    fn already_resolved(&self) -> bool {
        self.resolved
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Dependency {
    pub id: ItemId<Package>,
    pub enabled: Option<bool>,
}
