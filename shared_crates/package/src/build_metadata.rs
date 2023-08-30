use ambient_shared_types::asset::BuildAsset;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Version;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BuildMetadataError {
    #[error("Failed to parse build metadata")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BuildMetadata {
    pub ambient_version: Version,
    pub ambient_revision: String,
    pub client_component_paths: Vec<String>,
    pub server_component_paths: Vec<String>,
    #[serde(default)]
    pub last_build_time: Option<String>,
    #[serde(default)]
    pub settings: BuildSettings,
    #[serde(default)]
    pub asset: Vec<BuildAsset>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct BuildSettings {
    #[serde(default)]
    /// Build with optimizations.
    pub release: bool,
    #[serde(default)]
    /// Build the WASM files only.
    pub wasm_only: bool,
    #[serde(default)]
    /// Build with deployment in mind (i.e. ignore local dependencies).
    pub deploy: bool,
}

impl BuildMetadata {
    pub const FILENAME: &'static str = "metadata.toml";

    pub fn component_paths(&self, target: &str) -> &[String] {
        match target {
            "client" => &self.client_component_paths,
            "server" => &self.server_component_paths,
            _ => panic!("Unknown target `{}`", target),
        }
    }

    pub fn parse(contents: &str) -> Result<Self, BuildMetadataError> {
        Ok(toml::from_str(contents)?)
    }

    pub fn last_build_time(&self) -> chrono::ParseResult<Option<chrono::DateTime<chrono::Utc>>> {
        Ok(self
            .last_build_time
            .as_deref()
            .map(chrono::DateTime::parse_from_rfc3339)
            .transpose()?
            .map(|lbt| lbt.with_timezone(&chrono::Utc)))
    }
}
