use crate::global::{ProceduralMaterialHandle, ProceduralSamplerHandle, ProceduralTextureHandle};
use crate::internal::conversion::*;
use crate::internal::wit;

#[derive(Clone, Copy)]
pub struct Descriptor {
    pub base_color_map: ProceduralTextureHandle,
    pub normal_map: ProceduralTextureHandle,
    pub metallic_roughness_map: ProceduralTextureHandle,
    pub sampler: ProceduralSamplerHandle,
}

pub fn create(desc: &Descriptor) -> ProceduralMaterialHandle {
    wit::client_material::create(wit::client_material::Descriptor {
        base_color_map: desc.base_color_map.into_bindgen(),
        normal_map: desc.normal_map.into_bindgen(),
        metallic_roughness_map: desc.metallic_roughness_map.into_bindgen(),
        sampler: desc.sampler.into_bindgen(),
    })
    .from_bindgen()
}

pub fn destroy(handle: ProceduralMaterialHandle) {
    wit::client_material::destroy(handle.into_bindgen());
}
