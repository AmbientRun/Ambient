use std::collections::HashMap;

use anyhow::Context;
use yaml_rust::Yaml;

use crate::parse_unity_yaml;

/// A unity .fbx.meta file
#[derive(Debug, Default)]
pub struct ModelImporter {
    pub id_to_name: HashMap<i64, String>,
    pub use_file_scale: bool,
}
impl ModelImporter {
    pub fn from_string(data: &str) -> anyhow::Result<Self> {
        Self::from_yaml(&parse_unity_yaml(data)?[0])
    }
    pub fn from_yaml(doc: &Yaml) -> anyhow::Result<Self> {
        let model_importer = &doc["ModelImporter"];
        let id_to_name = if let Some(internal_id_to_name_table) = model_importer["internalIDToNameTable"].as_vec() {
            internal_id_to_name_table
                .iter()
                .map(|entry| {
                    let first = entry["first"].as_hash().unwrap().values().next().unwrap();
                    (first.as_i64().unwrap(), entry["second"].as_str().unwrap().to_string())
                })
                .collect::<HashMap<i64, String>>()
        } else {
            model_importer["fileIDToRecycleName"]
                .as_hash()
                .unwrap()
                .into_iter()
                .map(|(key, value)| (key.as_i64().unwrap(), value.as_str().unwrap().to_string()))
                .collect::<HashMap<i64, String>>()
        };
        Ok(Self { id_to_name, use_file_scale: model_importer["meshes"]["useFileScale"].as_i64().context("Can't read useFileScale")? != 0 })
    }
}
