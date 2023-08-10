use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::Version;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BuildMetadata {
    pub ambient_version: Version,
    pub ambient_revision: String,
    pub client_component_paths: Vec<String>,
    pub server_component_paths: Vec<String>,
    #[serde(default)]
    pub last_build_time: Option<String>,
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

    pub fn parse(contents: &str) -> anyhow::Result<Self> {
        toml::from_str(contents).context("failed to parse build metadata")
    }
}
