use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BuildAsset {
    #[serde(rename = "type")]
    pub type_: AssetType,
    pub input: Option<PathBuf>,
    pub output: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    AssetCrate,
    Prefab,
    ScriptBundle,
    Model,
    Image,
    Animation,
    Material,
    Collider,

    // These will be replaced by prefabs with components instead
    TerrainMaterial,
    Atmosphere,
    Biomes,

    /// Represents a vorbis backed file
    VorbisTrack,
    SoundGraph,
}
