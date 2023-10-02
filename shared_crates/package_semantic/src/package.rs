use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use ambient_package::{BuildMetadata, Identifier, Manifest, SnakeCaseIdentifier};
use ambient_std::path;
use thiserror::Error;
use url::Url;

use crate::{
    schema,
    util::{retrieve_file, retrieve_url, RetrieveError},
    Item, ItemData, ItemId, ItemType, ItemValue, Resolve, Scope, Semantic,
};
use semver::Version;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct PackageLocator {
    pub id: Identifier,
    pub version: Version,
    pub source: RetrievableFile,
}
impl PackageLocator {
    pub fn from_manifest(
        manifest: &Manifest,
        source: RetrievableFile,
        id_override: Option<Identifier>,
    ) -> Option<Self> {
        Some(Self {
            id: manifest
                .package
                .id
                .clone()
                .map(Identifier::from)
                .or(id_override)?,
            version: manifest.package.version.clone(),
            source,
        })
    }
}
impl Display for PackageLocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}[", self.id, self.version)?;
        match &self.source {
            RetrievableFile::Ambient(p) => write!(f, "ambient:{}", p.display()),
            RetrievableFile::Path(p) => write!(f, "path:{}", p.display()),
            RetrievableFile::Url(u) => write!(f, "url:{}", u),
            RetrievableFile::Deployment(d) => {
                write!(f, "deployment:{}/{}", d.id, d.path.display())
            }
        }?;
        write!(f, "]")
    }
}

#[derive(Error, Debug)]
pub enum GetError {
    #[error("Failed to find {0:?} in Ambient schema")]
    FailedToFindInAmbientSchema(PathBuf),
    #[error("Failed to read file")]
    ReadFileError(#[from] RetrieveError),
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
pub struct RetrievableDeployment {
    pub id: String,
    pub path: PathBuf,
}
impl RetrievableDeployment {
    pub fn url(&self) -> Url {
        let mut url = ambient_shared_types::urls::deployment_url(&self.id);
        url.push('/');

        let path = self.path.to_string_lossy();
        Url::parse(&url)
            .unwrap_or_else(|e| panic!("invalid deployment url {url}: {e}"))
            .join(path.as_ref())
            .unwrap_or_else(|e| panic!("invalid deployment url after join {url} + {path}: {e}"))
    }

    /// Retrieves the manifest from the cache if it exists, otherwise retrieves
    /// it from the deployment and caches it
    pub async fn retrieve_manifest(&self) -> Result<String, RetrieveError> {
        let cache_path = ambient_dirs::deployment_cache_path(&self.id).join(&self.path);
        if cache_path.exists() {
            return retrieve_file(&cache_path);
        }

        let manifest = retrieve_url(&self.url()).await?;

        std::fs::create_dir_all(cache_path.parent().unwrap()).ok();
        std::fs::write(&cache_path, &manifest).ok();

        Ok(manifest)
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
/// Paths should be to the manifest, not to the folder it's in
pub enum RetrievableFile {
    Ambient(PathBuf),
    Path(PathBuf),
    Url(Url),
    Deployment(RetrievableDeployment),
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
                    retrieve_url(&url).await?
                }
            }
            RetrievableFile::Url(url) => retrieve_url(url).await?,
            RetrievableFile::Deployment(d) => d.retrieve_manifest().await?,
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
            RetrievableFile::Deployment(old_deployment) => {
                RetrievableFile::Deployment(RetrievableDeployment {
                    id: old_deployment.id.clone(),
                    path: parent_join(&old_deployment.path, suffix)?,
                })
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
            Self::Deployment(deployment) => {
                #[cfg(target_os = "unknown")]
                {
                    let _deployment = deployment;
                    unimplemented!("getting the path of a deployment is not supported on web")
                }

                #[cfg(not(target_os = "unknown"))]
                Some(LocalOrRemote::Remote(deployment.url()))
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
            RetrievableFile::Deployment(d) => {
                write!(f, "deployment://{}/{}", d.id, d.path.display())
            }
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
