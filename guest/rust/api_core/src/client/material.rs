use crate::global::{ProceduralMaterialHandle, ProceduralSamplerHandle, ProceduralTextureHandle};
use crate::internal::conversion::*;
use crate::internal::wit;

#[derive(Clone, Copy)]
pub struct Descriptor {
    pub base_color_map: ProceduralTextureHandle,
    pub normal_map: ProceduralTextureHandle,
    pub metallic_roughness_map: ProceduralTextureHandle,
    pub sampler: ProceduralSamplerHandle,
    pub transparent: bool,
}

impl IntoBindgen for Descriptor {
    type Item = wit::client_material::Descriptor;

    fn into_bindgen(self) -> Self::Item {
        Self::Item {
            base_color_map: self.base_color_map.into_bindgen(),
            normal_map: self.normal_map.into_bindgen(),
            metallic_roughness_map: self.metallic_roughness_map.into_bindgen(),
            sampler: self.sampler.into_bindgen(),
            transparent: self.transparent,
        }
    }
}

pub fn create(desc: &Descriptor) -> ProceduralMaterialHandle {
    wit::client_material::create(desc.into_bindgen()).from_bindgen()
}

pub fn destroy(handle: ProceduralMaterialHandle) {
    wit::client_material::destroy(handle.into_bindgen());
}
