use std::{collections::HashSet, sync::Arc};

use ambient_std::asset_url::{AbsAssetUrl, AssetType};

#[derive(Debug, Clone)]
pub enum OutAssetContent {
    Content(AbsAssetUrl),
    Collection(Vec<String>),
}
impl OutAssetContent {
    pub fn is_collection(&self) -> bool {
        matches!(self, OutAssetContent::Collection(..))
    }
}

#[derive(Debug, Clone)]
pub enum OutAssetPreview {
    None,
    FromModel { url: AbsAssetUrl },
    Image { image: Arc<image::RgbaImage> },
}

#[derive(Debug, Clone)]
pub struct OutAsset {
    /// A unique id identifying this asset
    pub id: String,
    pub type_: AssetType,
    /// If this asset is not displayed in search results
    pub hidden: bool,
    pub name: String,
    pub tags: Vec<String>,
    /// Each entry in the vec is a category level, i.e.:
    /// self.categories[0].insert("Vehicles");
    /// self.categories[1].insert("Vehicles > Cars");
    pub categories: [HashSet<String>; 3],
    pub preview: OutAssetPreview,
    pub content: OutAssetContent,
    pub source: Option<AbsAssetUrl>,
}
pub fn asset_id_from_url(url: &AbsAssetUrl) -> String {
    slugify::slugify(&format!("{}{}", url.0.host_str().unwrap_or(""), url.0.path()), "", "_", None)
}
