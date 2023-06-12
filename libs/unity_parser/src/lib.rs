use anyhow::Context;
use glam::*;
use yaml_rust::{Yaml, YamlLoader};

pub mod asset;
pub mod mat;
pub mod model_importer;
pub mod prefab;

#[derive(Debug, Clone)]
pub struct UnityRef {
    pub file_id: i64,
    pub guid: Option<String>,
    pub type_: Option<i64>,
}
impl UnityRef {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        Ok(Self {
            file_id: yaml["fileID"].as_i64().context("fileID not i64")?,
            guid: yaml["guid"].as_str().map(|x| x.to_string()),
            type_: yaml["type"].as_i64(),
        })
    }
    pub fn is_remote(&self) -> bool {
        self.guid.is_some()
    }
    pub fn dump(&self, prefab: &prefab::PrefabFile, dump_game_obj: bool) -> yaml_rust::yaml::Hash {
        let mut out = prefab
            .objects
            .get(&self.file_id)
            .unwrap()
            .dump(prefab, dump_game_obj);
        out.insert(Yaml::String("id".to_string()), Yaml::Integer(self.file_id));
        out
    }
}
impl std::fmt::Display for UnityRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}_{:?}",
            self.file_id,
            self.guid.as_ref().map(|x| x as &str).unwrap_or(""),
            self.type_.unwrap_or_default()
        )
    }
}

fn quat_from_yaml(yaml: &Yaml) -> anyhow::Result<Quat> {
    Ok(Quat::from_xyzw(
        yaml["x"].as_float().context("Failed to parse Quat.x")? as f32,
        yaml["y"].as_float().context("Failed to parse Quat.y")? as f32,
        yaml["z"].as_float().context("Failed to parse Quat.z")? as f32,
        yaml["w"].as_float().context("Failed to parse Quat.w")? as f32,
    ))
}

fn vec3_from_yaml(yaml: &Yaml) -> anyhow::Result<Vec3> {
    Ok(Vec3 {
        x: yaml["x"].as_float().context("Failed to parse Vec3.x")? as f32,
        y: yaml["y"].as_float().context("Failed to parse Vec3.y")? as f32,
        z: yaml["z"].as_float().context("Failed to parse Vec3.z")? as f32,
    })
}

trait YamlExt {
    /// "1" will be treated as an integer by the yaml parser, even if we ask for a float, so this takes care of that
    fn as_float(&self) -> Option<f64>;
}
impl YamlExt for Yaml {
    fn as_float(&self) -> Option<f64> {
        self.as_f64().or_else(|| self.as_i64().map(|x| x as f64))
    }
}

pub fn parse_unity_yaml(data: &str) -> anyhow::Result<Vec<Yaml>> {
    let data = data
        .replace("!u!", "unity_object: ")
        .replacen("unity_object: ", "!u!", 1)
        .replace("--- unity_object: ", "---\nunity_object: ");
    let docs = YamlLoader::load_from_str(&data).context("Bad yaml")?;
    Ok(docs)
}
