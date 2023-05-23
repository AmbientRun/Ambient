use crate::global::ProceduralSamplerHandle;
use crate::internal::conversion::*;
use crate::internal::wit;

#[derive(Clone, Copy)]
pub enum FilterMode {
    Nearest,
    Linear,
}

impl IntoBindgen for FilterMode {
    type Item = wit::client_sampler::FilterMode;

    fn into_bindgen(self) -> Self::Item {
        match self {
            FilterMode::Nearest => Self::Item::Nearest,
            FilterMode::Linear => Self::Item::Linear,
        }
    }
}

#[derive(Clone, Copy)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

impl IntoBindgen for AddressMode {
    type Item = wit::client_sampler::AddressMode;

    fn into_bindgen(self) -> Self::Item {
        match self {
            AddressMode::ClampToEdge => Self::Item::ClampToEdge,
            AddressMode::Repeat => Self::Item::Repeat,
            AddressMode::MirrorRepeat => Self::Item::MirrorRepeat,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Descriptor {
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
}

impl IntoBindgen for Descriptor {
    type Item = wit::client_sampler::Descriptor;

    fn into_bindgen(self) -> Self::Item {
        Self::Item {
            address_mode_u: self.address_mode_u.into_bindgen(),
            address_mode_v: self.address_mode_v.into_bindgen(),
            address_mode_w: self.address_mode_w.into_bindgen(),
            mag_filter: self.mag_filter.into_bindgen(),
            min_filter: self.min_filter.into_bindgen(),
            mipmap_filter: self.mipmap_filter.into_bindgen(),
        }
    }
}

pub fn create(desc: &Descriptor) -> ProceduralSamplerHandle {
    wit::client_sampler::create(desc.into_bindgen()).from_bindgen()
}

pub fn destroy(handle: ProceduralSamplerHandle) {
    wit::client_sampler::destroy(handle.into_bindgen());
}
