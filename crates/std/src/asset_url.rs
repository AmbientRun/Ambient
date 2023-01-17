use std::{
    marker::PhantomData, path::{Path, PathBuf}, sync::Arc
};

use anyhow::Context;
use convert_case::{Case, Casing};
use percent_encoding::percent_decode_str;
use rand::seq::SliceRandom;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{
    de::{DeserializeOwned, Visitor}, Deserialize, Deserializer, Serialize, Serializer
};
use url::Url;

use crate::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt}, download_asset::{download, AssetsCacheDir}, Cb
};

/// This is a thin wrapper around Url, which is guaranteed to always
/// be an absolute url (including when pointing to a local file).
///
/// It's got a custom Debug implementation which just prints the url,
/// which makes it useful in asset keys
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AbsAssetUrl(pub Url);
impl std::fmt::Debug for AbsAssetUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}
impl std::fmt::Display for AbsAssetUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}
impl AbsAssetUrl {
    /// This will also resolve relative local paths
    pub fn parse(url: impl AsRef<str>) -> anyhow::Result<Self> {
        match Url::parse(url.as_ref()) {
            Ok(url) => Ok(Self(url)),
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                Ok(Self(Url::parse(&format!("file://{}/{}", std::env::current_dir().unwrap().to_str().unwrap(), url.as_ref()))?))
            }
            Err(err) => Err(err.into()),
        }
    }
    pub fn from_file_path(path: impl AsRef<Path>) -> Self {
        if path.as_ref().is_absolute() {
            Self(Url::from_file_path(path).unwrap())
        } else {
            let path = std::env::current_dir().unwrap().join(path);
            Self(Url::from_file_path(path).unwrap())
        }
    }
    pub fn from_directory_path(path: impl AsRef<Path>) -> Self {
        if path.as_ref().is_absolute() {
            Self(Url::from_directory_path(path).unwrap())
        } else {
            let path = std::env::current_dir().unwrap().join(path);
            Self(Url::from_directory_path(path).unwrap())
        }
    }
    pub fn relative_cache_path(&self) -> String {
        self.0.to_string().replace("://", "/")
    }
    pub fn absolute_cache_path(&self, assets: &AssetCache) -> PathBuf {
        AssetsCacheDir.get(assets).join(self.relative_cache_path())
    }
    /// This is always lowercase
    pub fn extension(&self) -> Option<String> {
        self.0.path().rsplit_once('.').map(|(_, ext)| ext.to_string().to_lowercase())
    }
    /// This is always lowercase
    pub fn extension_is(&self, extension: impl AsRef<str>) -> bool {
        self.extension() == Some(extension.as_ref().to_string())
    }
    /// Appends the extension: test.png -> test.png.hello
    pub fn add_extension(&self, extension: &str) -> Self {
        let mut url = self.0.clone();
        url.set_path(&format!("{}.{}", url.path(), extension));
        Self(url)
    }
    pub fn to_file_path(&self) -> anyhow::Result<Option<PathBuf>> {
        if self.0.scheme() == "file" {
            match self.0.to_file_path() {
                Ok(path) => Ok(Some(path)),
                Err(_) => Err(anyhow::anyhow!("Invalid file url: {:?}", self)),
            }
        } else {
            Ok(None)
        }
    }
    pub fn resolve(&self, url_or_relative_path: impl AsRef<str>) -> Result<Self, url::ParseError> {
        AssetUrl::parse(url_or_relative_path)?.resolve(self)
    }
    /// This appends [path] to the current path, with a `/` joining them
    pub fn push(&self, path: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Ok(AbsAssetUrl(self.as_directory().0.join(path.as_ref())?))
    }
    /// This joins the current url with a relative path. See https://docs.rs/url/latest/url/struct.Url.html#method.join for details how it works
    pub fn join(&self, path: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Ok(AbsAssetUrl(self.0.join(path.as_ref())?))
    }
    /// Returns the decoded path
    pub fn path(&self) -> RelativePathBuf {
        RelativePathBuf::from(&percent_decode_str(self.0.path()).decode_utf8().unwrap())
    }
    pub fn set_path(&mut self, path: impl AsRef<str>) {
        self.0.set_path(path.as_ref());
    }
    pub fn relative_path(&self, path: impl AsRef<RelativePath>) -> RelativePathBuf {
        RelativePathBuf::from(self.0.path()).relative(path)
    }
    pub fn is_directory(&self) -> bool {
        self.0.path().ends_with("/")
    }
    /// Ensures that this url ends with `/`, which is interpreted as a "directory" by the Url package
    pub fn as_directory(&self) -> Self {
        let mut res = self.clone();
        if !res.is_directory() {
            res.set_path(&format!("{}/", res.path()));
        }
        res
    }
    /// Ensures that this url doesn't end with `/`, which is interpreted as a "directory" by the Url package
    pub fn as_file(&self) -> Self {
        let mut res = self.clone();
        if res.is_directory() {
            res.set_path(&res.path().as_str()[0..(res.path().as_str().len() - 1)]);
        }
        res
    }
    /// For a/b/c.png this returns b
    pub fn last_dir_name(&self) -> Option<&str> {
        let mut segs = self.0.path_segments()?.rev();
        segs.next()?; // discard
        segs.next()
    }
    pub async fn download_bytes(&self, assets: &AssetCache) -> anyhow::Result<Vec<u8>> {
        if let Some(path) = self.to_file_path()? {
            Ok(tokio::fs::read(path).await.context(format!("Failed to read file at: {:}", self.0))?)
        } else {
            Ok(download(&assets, self.0.clone(), |resp| async { Ok(resp.bytes().await?) }).await?.to_vec())
        }
    }
    pub async fn download_string(&self, assets: &AssetCache) -> anyhow::Result<String> {
        if let Some(path) = self.to_file_path()? {
            Ok(tokio::fs::read_to_string(path).await.context(format!("Failed to read file at: {:}", self.0))?)
        } else {
            Ok(download(&assets, self.0.clone(), |resp| async { Ok(resp.text().await?) }).await?)
        }
    }
    pub async fn download_json<T: DeserializeOwned>(&self, assets: &AssetCache) -> anyhow::Result<T> {
        if let Some(path) = self.to_file_path()? {
            let content: Vec<u8> = tokio::fs::read(path).await.context(format!("Failed to read file at: {:}", self.0))?;
            Ok(serde_json::from_slice(&content)?)
        } else {
            Ok(download(&assets, self.0.clone(), |resp| async { Ok(resp.json::<T>().await?) }).await?)
        }
    }
    pub async fn download_toml<T: DeserializeOwned>(&self, assets: &AssetCache) -> anyhow::Result<T> {
        let content = self.download_bytes(assets).await?;
        Ok(toml::from_slice(&content)?)
    }
}
impl From<PathBuf> for AbsAssetUrl {
    fn from(value: PathBuf) -> Self {
        let value = if value.is_absolute() { value } else { std::env::current_dir().unwrap().join(value) };
        Self(Url::from_file_path(value).unwrap())
    }
}

#[test]
fn test_abs_asset_url() {
    assert_eq!(AbsAssetUrl::parse("http://t.c/hello").unwrap().as_directory().to_string(), "http://t.c/hello/");
    assert_eq!(AbsAssetUrl::parse("http://t.c/hello/").unwrap().as_directory().to_string(), "http://t.c/hello/");
    assert_eq!(AbsAssetUrl::parse("http://t.c/hello").unwrap().as_file().to_string(), "http://t.c/hello");
    assert_eq!(AbsAssetUrl::parse("http://t.c/hello/").unwrap().as_file().to_string(), "http://t.c/hello");

    assert_eq!(AbsAssetUrl::parse("http://t.c/a/b/c.png").unwrap().last_dir_name(), Some("b"));
}

/// This is either an absolute url (which can also be an absolute file:// url),
/// or a relative path which needs to be resolved
///
/// When serialized, this serializes to a string
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum AssetUrl {
    Absolute(AbsAssetUrl),
    Relative(RelativePathBuf),
}
impl AssetUrl {
    pub fn parse(url_or_relative_path: impl AsRef<str>) -> Result<Self, url::ParseError> {
        match Url::parse(url_or_relative_path.as_ref()) {
            Ok(url) => Ok(Self::Absolute(AbsAssetUrl(url))),
            Err(url::ParseError::RelativeUrlWithoutBase) => Ok(Self::Relative(url_or_relative_path.as_ref().into())),
            Err(err) => Err(err),
        }
    }
    /// This is always lowercase
    pub fn extension(&self) -> Option<String> {
        match self {
            AssetUrl::Absolute(url) => url.extension(),
            AssetUrl::Relative(path) => path.extension().map(|x| x.to_string().to_lowercase()),
        }
    }
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> Result<AbsAssetUrl, url::ParseError> {
        match self {
            AssetUrl::Absolute(url) => Ok(url.clone()),
            AssetUrl::Relative(path) => Ok(AbsAssetUrl(base_url.0.join(path.as_str())?)),
        }
    }
    pub fn path(&self) -> &str {
        match self {
            AssetUrl::Absolute(url) => url.0.path(),
            AssetUrl::Relative(path) => path.as_str(),
        }
    }
    pub fn join(&self, path: impl Into<RelativePathBuf>) -> Result<Self, url::ParseError> {
        let path: RelativePathBuf = path.into();
        match self {
            AssetUrl::Absolute(url) => Ok(Self::Absolute(AbsAssetUrl(url.0.join(path.as_str())?))),
            AssetUrl::Relative(p) => Ok(Self::Relative(p.join(path))),
        }
    }
    pub fn parent(&self) -> Option<Self> {
        match self {
            AssetUrl::Absolute(url) => Some(Self::Absolute(AbsAssetUrl(url.0.join("..").ok()?))),
            AssetUrl::Relative(path) => Some(Self::Relative(path.parent()?.to_relative_path_buf())),
        }
    }
    pub fn abs(&self) -> Option<AbsAssetUrl> {
        match self {
            AssetUrl::Absolute(url) => Some(url.clone()),
            AssetUrl::Relative(_) => None,
        }
    }
    pub fn unwrap_abs(self) -> AbsAssetUrl {
        self.abs().expect("This AssetUrl hasn't been resolved yet")
    }
}
impl From<RelativePathBuf> for AssetUrl {
    fn from(value: RelativePathBuf) -> Self {
        Self::Relative(value)
    }
}
impl From<Url> for AssetUrl {
    fn from(value: Url) -> Self {
        Self::Absolute(AbsAssetUrl(value))
    }
}
impl From<AbsAssetUrl> for AssetUrl {
    fn from(value: AbsAssetUrl) -> Self {
        Self::Absolute(value)
    }
}
impl From<Arc<AbsAssetUrl>> for AssetUrl {
    fn from(value: Arc<AbsAssetUrl>) -> Self {
        Self::Absolute((*value).clone())
    }
}
impl std::fmt::Debug for AssetUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absolute(arg0) => write!(f, "{arg0}"),
            Self::Relative(arg0) => write!(f, "{arg0}"),
        }
    }
}
impl std::fmt::Display for AssetUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absolute(arg0) => write!(f, "{arg0}"),
            Self::Relative(arg0) => write!(f, "{arg0}"),
        }
    }
}
impl Default for AssetUrl {
    fn default() -> Self {
        Self::Relative(Default::default())
    }
}
impl Serialize for AssetUrl {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for AssetUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AssetUrlVisitor;

        impl<'de> Visitor<'de> for AssetUrlVisitor {
            type Value = AssetUrl;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AssetUrl")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                AssetUrl::parse(v).map_err(|err| E::custom(format!("Bad asset url format: {err:?}")))
            }
        }

        deserializer.deserialize_str(AssetUrlVisitor)
    }
}

/// This is a typed wrapper for AssetUrl. By adding the type information,
/// we can render a correct editor ui when used in a ui context.
///
/// An TypedAssetUrl will be rendered with a Browse button next to it in the UI,
/// which takes you to the AssetBrowser. See `elements_ui/src/asset_url` for
/// the UI implementation and `dims_asset_browser` for the asset browser implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypedAssetUrl<T: GetAssetType>(pub AssetUrl, PhantomData<T>);

impl<T: GetAssetType> TypedAssetUrl<T> {
    pub fn parse(url_or_relative_path: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Ok(Self(AssetUrl::parse(url_or_relative_path)?, PhantomData))
    }
    pub fn new(url: impl Into<AssetUrl>) -> Self {
        Self(url.into(), PhantomData)
    }
    pub fn asset_type(&self) -> AssetType {
        T::asset_type()
    }
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> Result<AbsAssetUrl, url::ParseError> {
        self.0.resolve(base_url)
    }
    pub fn join<Y: GetAssetType>(&self, path: impl Into<RelativePathBuf>) -> Result<TypedAssetUrl<Y>, url::ParseError> {
        Ok(TypedAssetUrl::<Y>(self.0.join(path)?, PhantomData))
    }
    pub fn parent<Y: GetAssetType>(&self) -> Option<TypedAssetUrl<Y>> {
        Some(TypedAssetUrl::<Y>(self.0.parent()?, PhantomData))
    }
    pub fn abs(&self) -> Option<AbsAssetUrl> {
        self.0.abs()
    }
    pub fn unwrap_abs(self) -> AbsAssetUrl {
        self.0.unwrap_abs()
    }
}
impl<T: GetAssetType> std::fmt::Display for TypedAssetUrl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<T: GetAssetType> PartialEq for TypedAssetUrl<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.asset_type() == other.asset_type()
    }
}
impl<T: GetAssetType> Eq for TypedAssetUrl<T> {}
impl<T: GetAssetType> Default for TypedAssetUrl<T> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}
impl<T: GetAssetType> From<RelativePathBuf> for TypedAssetUrl<T> {
    fn from(value: RelativePathBuf) -> Self {
        Self(AssetUrl::Relative(value), PhantomData)
    }
}
impl<T: GetAssetType> From<AbsAssetUrl> for TypedAssetUrl<T> {
    fn from(value: AbsAssetUrl) -> Self {
        Self(AssetUrl::Absolute(value), PhantomData)
    }
}
impl<T: GetAssetType> From<AssetUrl> for TypedAssetUrl<T> {
    fn from(value: AssetUrl) -> Self {
        Self(value, PhantomData)
    }
}

/// Same as TypedAssetUrl, except it supports working with collections of asset urls
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetUrlCollection<T: GetAssetType>(pub Vec<AssetUrl>, PhantomData<T>);
impl<T: GetAssetType> AssetUrlCollection<T> {
    pub fn new(values: Vec<AssetUrl>) -> Self {
        Self(values, PhantomData)
    }
    pub fn asset_type(&self) -> AssetType {
        T::asset_type()
    }
}
impl<T: GetAssetType> Default for AssetUrlCollection<T> {
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}
impl<T: GetAssetType> PartialEq for AssetUrlCollection<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.asset_type() == other.asset_type()
    }
}
impl<T: GetAssetType> Eq for AssetUrlCollection<T> {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    AssetCrate,
    Object,
    ScriptBundle,
    Model,
    Image,
    Animation,
    Material,
    Collider,

    // These will be replaced by Objects with components instead
    TerrainMaterial,
    Atmosphere,
    Biomes,

    /// Represents a vorbis backed file
    VorbisTrack,
    SoundGraph,
}

impl AssetType {
    pub fn to_snake_case(&self) -> String {
        format!("{self:?}").to_case(Case::Snake)
    }
}

pub trait GetAssetType: std::fmt::Debug + Clone + Sync + Send {
    fn asset_type() -> AssetType;
}

#[derive(Debug, Clone)]
pub struct ModelCrateAssetType;
impl GetAssetType for ModelCrateAssetType {
    fn asset_type() -> AssetType {
        AssetType::AssetCrate
    }
}
impl TypedAssetUrl<ModelCrateAssetType> {
    pub fn model(&self) -> TypedAssetUrl<ModelAssetType> {
        self.join("models/main.json").unwrap()
    }
    pub fn object(&self) -> TypedAssetUrl<ObjectAssetType> {
        self.join("objects/main.json").unwrap()
    }
    pub fn collider(&self) -> TypedAssetUrl<ColliderAssetType> {
        self.join("colliders/main.json").unwrap()
    }
    pub fn animation(&self, id: &str) -> TypedAssetUrl<AnimationAssetType> {
        self.join(format!("animations/{id}.json")).unwrap()
    }
    pub fn material(&self, id: &str) -> TypedAssetUrl<MaterialAssetType> {
        self.join(format!("materials/{id}.json")).unwrap()
    }
    pub fn image(&self, id: &str) -> TypedAssetUrl<ImageAssetType> {
        self.join(format!("images/{id}.json")).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ObjectAssetType;
impl GetAssetType for ObjectAssetType {
    fn asset_type() -> AssetType {
        AssetType::Object
    }
}
impl TypedAssetUrl<ObjectAssetType> {
    pub fn asset_crate(&self) -> Option<TypedAssetUrl<AssetCrateAssetType>> {
        self.join("..").ok()
    }
}
pub type ObjectRef = TypedAssetUrl<ObjectAssetType>;

#[derive(Debug, Clone)]
pub struct ModelAssetType;
impl GetAssetType for ModelAssetType {
    fn asset_type() -> AssetType {
        AssetType::Model
    }
}
impl TypedAssetUrl<ModelAssetType> {
    pub fn asset_crate(&self) -> Option<TypedAssetUrl<AssetCrateAssetType>> {
        self.join("..").ok()
    }
}

#[derive(Debug, Clone)]
pub struct AnimationAssetType;
impl GetAssetType for AnimationAssetType {
    fn asset_type() -> AssetType {
        AssetType::Animation
    }
}
impl TypedAssetUrl<AnimationAssetType> {
    pub fn asset_crate(&self) -> Option<TypedAssetUrl<AssetCrateAssetType>> {
        self.join("..").ok()
    }
}

#[derive(Debug, Clone)]
pub struct ImageAssetType;
impl GetAssetType for ImageAssetType {
    fn asset_type() -> AssetType {
        AssetType::Image
    }
}

#[derive(Debug, Clone)]
pub struct MaterialAssetType;
impl GetAssetType for MaterialAssetType {
    fn asset_type() -> AssetType {
        AssetType::Material
    }
}
impl TypedAssetUrl<MaterialAssetType> {
    pub fn model_crate(&self) -> Option<TypedAssetUrl<ModelCrateAssetType>> {
        Some(self.join("..").ok()?)
    }
}

#[derive(Debug, Clone)]
pub struct SoundAssetType;
impl GetAssetType for SoundAssetType {
    fn asset_type() -> AssetType {
        AssetType::VorbisTrack
    }
}

#[derive(Debug, Clone)]
pub struct ColliderAssetType;
impl GetAssetType for ColliderAssetType {
    fn asset_type() -> AssetType {
        AssetType::Collider
    }
}

#[test]
fn test_join() {
    let obj = TypedAssetUrl::<ObjectAssetType>::parse("https://playdims.com/api/v1/assetdb/crates/RxH7k2ox5Ug6DNcqJhta/1.7.0/quixel_groundcover_wcwmchzja_2k_3dplant_ms_wcwmchzja_json0/objects/main.json").unwrap();
    let crat = obj.model_crate().unwrap();
    assert_eq!(
        crat.to_string(),
        "https://playdims.com/api/v1/assetdb/crates/RxH7k2ox5Ug6DNcqJhta/1.7.0/quixel_groundcover_wcwmchzja_2k_3dplant_ms_wcwmchzja_json0/"
    );
    let model = crat.model();
    assert_eq!(model.to_string(), "https://playdims.com/api/v1/assetdb/crates/RxH7k2ox5Ug6DNcqJhta/1.7.0/quixel_groundcover_wcwmchzja_2k_3dplant_ms_wcwmchzja_json0/models/main.json");
}

/// Invoking this method will show a UI for selecting an asset from the asset db, and will then return the result of that
#[derive(Clone, Debug)]
pub struct SelectAssetCbKey;
impl SyncAssetKey<Cb<dyn Fn(AssetType, Box<dyn FnOnce(SelectedAsset<String>) + Sync + Send>) + Sync + Send>> for SelectAssetCbKey {}

#[derive(Debug)]
pub enum SelectedAsset<T> {
    None,
    Asset { content: T, name: String },
    Collection { content: Vec<T>, name: String },
}
impl<T> SelectedAsset<T> {
    pub fn map<X>(self, map: impl Fn(T) -> X) -> SelectedAsset<X> {
        match self {
            SelectedAsset::None => SelectedAsset::None,
            SelectedAsset::Asset { content, name } => SelectedAsset::Asset { content: map(content), name },
            SelectedAsset::Collection { content, name } => {
                SelectedAsset::Collection { content: content.into_iter().map(map).collect(), name }
            }
        }
    }
    pub fn name(&self) -> Option<&str> {
        match self {
            SelectedAsset::None => None,
            SelectedAsset::Asset { name, .. } => Some(name),
            SelectedAsset::Collection { name, .. } => Some(name),
        }
    }
    pub fn random(&self) -> Option<&T> {
        self.all().choose(&mut rand::thread_rng()).copied()
    }
    pub fn all(&self) -> Vec<&T> {
        match self {
            SelectedAsset::None => vec![],
            SelectedAsset::Asset { content, .. } => vec![content],
            SelectedAsset::Collection { content, .. } => content.iter().collect(),
        }
    }
}

pub fn select_asset(assets: &AssetCache, asset_type: AssetType, cb: impl FnOnce(SelectedAsset<String>) + Sync + Send + 'static) {
    let func = SelectAssetCbKey.get(assets);
    (func)(asset_type, Box::new(cb));
}

pub fn select_asset_json<T: DeserializeOwned + Send + Sync + 'static>(
    assets: &AssetCache,
    asset_type: AssetType,
    cb: impl FnOnce(SelectedAsset<T>) + Sync + Send + 'static,
) {
    let func = SelectAssetCbKey.get(assets);
    (func)(asset_type, Box::new(move |content| cb(content.map(|content| serde_json::from_str(&content).unwrap()))));
}
