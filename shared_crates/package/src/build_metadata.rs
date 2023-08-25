use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Version;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BuildMetadataError {
    #[error("failed to parse build metadata")]
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
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BuildSettings {
    pub optimize: bool,
    pub build_wasm_only: bool,
    pub building_for_deploy: bool,
}
impl Default for BuildSettings {
    fn default() -> Self {
        Self {
            optimize: false,
            build_wasm_only: false,
            building_for_deploy: false,
        }
    }
}
