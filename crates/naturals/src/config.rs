use std::sync::Arc;

use ambient_editor_derive::ElementEditor;
use ambient_model::Model;
use ambient_std::{
    asset_cache::AsyncAssetKeyExt,
    asset_url::{AssetUrlCollection, ObjectAssetType},
    download_asset::AssetError,
    include_file,
};
use glam::{vec3, Vec3, Vec4};
use serde::{Deserialize, Serialize};

fn true_value() -> bool {
    true
}
fn white_value() -> RGB {
    RGB { r: 1., g: 1., b: 1. }
}

#[derive(Clone, Debug, Serialize, Deserialize, ElementEditor)]
pub struct RGB {
    #[editor(slider, min = 0., max = 2.)]
    pub r: f32,
    #[editor(slider, min = 0., max = 2.)]
    pub g: f32,
    #[editor(slider, min = 0., max = 2.)]
    pub b: f32,
}
impl From<RGB> for Vec3 {
    fn from(rgb: RGB) -> Self {
        vec3(rgb.r, rgb.g, rgb.b)
    }
}
impl From<RGB> for Vec4 {
    fn from(rgb: RGB) -> Self {
        vec3(rgb.r, rgb.g, rgb.b).extend(1.)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, ElementEditor)]
pub struct NaturalLayer {
    #[editor(slider, min = 0.05, max = 5.)]
    pub grid_size: f32,
    pub elements: Vec<NaturalElement>,
}
#[derive(Clone, Debug, Serialize, Deserialize, ElementEditor)]
pub struct NaturalElement {
    #[serde(default = "true_value")]
    pub enabled: bool,
    pub models: Vec<AssetUrlCollection<ObjectAssetType>>,

    #[serde(default = "white_value")]
    pub color: RGB,

    #[editor(slider, min = 0.1, max = 10., logarithmic)]
    pub scale_min: f32,
    #[editor(slider, min = 0.1, max = 30., logarithmic)]
    pub scale_max: f32,
    #[editor(slider, min = 0.1, max = 10.)]
    pub scale_power: f32,
    #[serde(default)]
    #[editor(slider, min = -3.15169, max = 3.15169)]
    pub rotation_x: f32,
    #[serde(default)]
    #[editor(slider, min = -3.15169, max = 3.15169)]
    pub rotation_y: f32,
    #[serde(default)]
    #[editor(slider, min = -3.15169, max = 3.15169)]
    pub rotation_z: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 1.)]
    pub rotation_z_jitter: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 1.)]
    pub rotation_xy_jitter: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 1.)]
    pub rotation_straightness: f32,
    #[serde(default)]
    #[editor(slider, min = -20., max = 5.)]
    pub position_normal_offset: f32,
    #[serde(default)]
    #[editor(slider, min = -20., max = 5.)]
    pub position_z_offset: f32,
    #[serde(default)]
    #[editor(slider, min = 0., max = 6.)]
    pub normal_miplevel: f32,
    #[serde(default = "cluster_noise_scale_default")]
    #[editor(slider, min = 0.01, max = 10., logarithmic)]
    pub cluster_noise_scale: f32,
    pub soil_depth: NaturalCurve,
    pub elevation: NaturalCurve,
    pub water_depth: NaturalCurve,
    pub steepness: NaturalCurve,
    #[serde(default)]
    pub cluster_noise: NaturalCurve,
}
fn cluster_noise_scale_default() -> f32 {
    1.
}
impl Default for NaturalElement {
    fn default() -> Self {
        Self {
            enabled: true,
            models: Default::default(),
            color: white_value(),
            scale_min: 1.,
            scale_max: 1.,
            scale_power: 1.,
            rotation_x: Default::default(),
            rotation_y: Default::default(),
            rotation_z: Default::default(),
            rotation_z_jitter: Default::default(),
            rotation_xy_jitter: Default::default(),
            rotation_straightness: Default::default(),
            position_normal_offset: Default::default(),
            position_z_offset: Default::default(),
            normal_miplevel: Default::default(),
            cluster_noise_scale: 1.,
            soil_depth: Default::default(),
            elevation: Default::default(),
            water_depth: Default::default(),
            steepness: Default::default(),
            cluster_noise: Default::default(),
        }
    }
}

pub type BoxModelKey = Box<dyn AsyncAssetKeyExt<Result<Arc<Model>, AssetError>>>;

#[derive(Clone, Debug, Serialize, Deserialize, ElementEditor)]
pub enum NaturalCurve {
    Constant {
        #[editor(slider, min = 0., max = 1.)]
        value: f32,
    },
    Interpolate {
        x0: f32,
        x1: f32,
        #[editor(slider, min = 0., max = 1.)]
        y0: f32,
        #[editor(slider, min = 0., max = 1.)]
        y1: f32,
    },
    InterpolateClamped {
        x0: f32,
        x1: f32,
        #[editor(slider, min = 0., max = 1.)]
        y0: f32,
        #[editor(slider, min = 0., max = 1.)]
        y1: f32,
    },
    SmoothStep {
        x0: f32,
        x1: f32,
        #[editor(slider, min = 0., max = 1.)]
        y0: f32,
        #[editor(slider, min = 0., max = 1.)]
        y1: f32,
    },
    BellCurve {
        center: f32,
        width: f32,
        #[editor(slider, min = 0., max = 1.)]
        y0: f32,
        #[editor(slider, min = 0., max = 1.)]
        y1: f32,
    },
}
impl Default for NaturalCurve {
    fn default() -> Self {
        Self::Constant { value: 1. }
    }
}

pub enum NaturalsPreset {
    Mountains,
    Desert,
    LowPerformanceMode,
}

pub fn get_default_natural_layers(preset: NaturalsPreset) -> Vec<NaturalLayer> {
    match preset {
        NaturalsPreset::Mountains => serde_json::from_str(&include_file!("mountains.json")).unwrap(),
        NaturalsPreset::Desert => serde_json::from_str(&include_file!("desert.json")).unwrap(),
        NaturalsPreset::LowPerformanceMode => serde_json::from_str(&include_file!("low_performance.json")).unwrap(),
    }
}
