use std::marker::PhantomData;

use convert_case::{Case, Casing};
use rand::seq::SliceRandom;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt}, Cb
};

/// This is a wrapper for a URL (pointing to an asset)
///
/// An AssetUrl will be rendered with a Browse button next to it in the UI,
/// which takes you to the AssetBrowser. See `elements_ui/src/asset_url` for
/// the UI implementation and `dims_asset_browser` for the asset browser implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetUrl<T: GetAssetType> {
    pub url: String,
    pub display_name: Option<String>,
    #[serde(skip)]
    pub asset_type: PhantomData<T>,
}

impl<T: GetAssetType> AssetUrl<T> {
    pub fn from_url(url: impl Into<String>) -> Self {
        Self { url: url.into(), display_name: None, asset_type: PhantomData }
    }
    pub fn new(download_url: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self { url: download_url.into(), display_name: Some(display_name.into()), asset_type: PhantomData }
    }
    fn new2(download_url: impl Into<String>, display_name: &Option<String>) -> Self {
        Self { url: download_url.into(), display_name: display_name.clone(), asset_type: PhantomData }
    }
}
impl<T: GetAssetType> Default for AssetUrl<T> {
    fn default() -> Self {
        Self { url: Default::default(), display_name: None, asset_type: PhantomData }
    }
}
impl<T: GetAssetType> PartialEq for AssetUrl<T> {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.asset_type == other.asset_type
    }
}
impl<T: GetAssetType> Eq for AssetUrl<T> {}
impl<T: Into<String>, X: GetAssetType> From<T> for AssetUrl<X> {
    fn from(s: T) -> Self {
        Self::from_url(s)
    }
}

/// Same as AssetUrl, except it supports working with collections of asset urls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetUrlCollection<T: GetAssetType> {
    pub urls: Vec<String>,
    pub display_name: Option<String>,
    #[serde(skip)]
    pub asset_type: PhantomData<T>,
}
impl<T: GetAssetType> Default for AssetUrlCollection<T> {
    fn default() -> Self {
        Self { urls: Default::default(), display_name: None, asset_type: PhantomData }
    }
}
impl<T: GetAssetType> PartialEq for AssetUrlCollection<T> {
    fn eq(&self, other: &Self) -> bool {
        self.urls == other.urls && self.asset_type == other.asset_type
    }
}
impl<T: GetAssetType> Eq for AssetUrlCollection<T> {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
        format!("{:?}", self).to_case(Case::Snake)
    }
}

pub trait GetAssetType: std::fmt::Debug + Clone + Sync + Send {
    fn asset_type() -> AssetType;
}

#[derive(Debug, Clone)]
pub struct AssetCrateAssetType;
impl GetAssetType for AssetCrateAssetType {
    fn asset_type() -> AssetType {
        AssetType::AssetCrate
    }
}
impl AssetUrl<AssetCrateAssetType> {
    pub fn model(&self) -> AssetUrl<ModelAssetType> {
        AssetUrl::<ModelAssetType>::new2(format!("{}/models/main.json", self.url), &self.display_name)
    }
    pub fn collider(&self) -> AssetUrl<ColliderAssetType> {
        AssetUrl::<ColliderAssetType>::new2(format!("{}/colliders/main.json", self.url), &self.display_name)
    }
}

#[derive(Debug, Clone)]
pub struct ObjectAssetType;
impl GetAssetType for ObjectAssetType {
    fn asset_type() -> AssetType {
        AssetType::Object
    }
}

#[derive(Debug, Clone)]
pub struct ModelAssetType;
impl GetAssetType for ModelAssetType {
    fn asset_type() -> AssetType {
        AssetType::Model
    }
}
impl AssetUrl<ModelAssetType> {
    pub fn asset_crate(&self) -> Option<AssetUrl<AssetCrateAssetType>> {
        let (start, _) = self.url.split_once("/models/")?;
        Some(AssetUrl::<AssetCrateAssetType>::new2(start, &self.display_name))
    }
}

#[derive(Debug, Clone)]
pub struct AnimationAssetType;
impl GetAssetType for AnimationAssetType {
    fn asset_type() -> AssetType {
        AssetType::Animation
    }
}
impl AssetUrl<AnimationAssetType> {
    pub fn asset_crate(&self) -> Option<AssetUrl<AssetCrateAssetType>> {
        let (start, _) = self.url.split_once("/animations/")?;
        Some(AssetUrl::<AssetCrateAssetType>::new2(start, &self.display_name))
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
