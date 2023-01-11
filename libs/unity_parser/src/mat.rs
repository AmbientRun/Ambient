use anyhow::Context;
use yaml_rust::Yaml;

use crate::{parse_unity_yaml, UnityRef, YamlExt};

/// A unity .mat file
#[derive(Debug, Default)]
pub struct Material {
    pub main_tex: Option<UnityRef>,
    pub bump_map: Option<UnityRef>,
    pub metallic_r_ao_g_smothness_a: Option<UnityRef>,
    pub metallic_gloss_map: Option<UnityRef>,
    pub occlusion_map: Option<UnityRef>,
    pub alpha_cutoff: Option<f32>,
}
impl Material {
    pub fn from_string(data: &str) -> anyhow::Result<Self> {
        Self::from_yaml(&parse_unity_yaml(data)?[0])
    }
    pub fn from_yaml(doc: &Yaml) -> anyhow::Result<Self> {
        let mat = &doc["Material"];
        let mut res = Material { alpha_cutoff: mat["m_Floats"]["_Cutoff"].as_float().map(|x| x as f32), ..Default::default() };
        for tex in mat["m_SavedProperties"]["m_TexEnvs"].as_vec().context("m_TexEnvs not a vec")? {
            let (key, value) = tex.as_hash().unwrap().iter().next().unwrap();
            let ref_ = UnityRef::from_yaml(&value["m_Texture"])?;
            match key.as_str().context("Key not a str")? {
                "_MainTex" => {
                    res.main_tex = Some(ref_);
                }
                "_BumpMap" => {
                    res.bump_map = Some(ref_);
                }
                "_MetalicRAOGSmothnessA" => {
                    res.metallic_r_ao_g_smothness_a = Some(ref_);
                }
                "_MetallicGlossMap" => {
                    res.metallic_gloss_map = Some(ref_);
                }
                "_OcclusionMap" => {
                    res.occlusion_map = Some(ref_);
                }
                _ => {}
            }
        }
        Ok(res)
    }
}
