use crate::internal::wit;

#[derive(Clone, Copy)]
pub enum FilterMode {
    Nearest,
    Linear,
}

#[derive(Clone, Copy)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
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

pub fn create(desc: &Descriptor) -> String {
    let address_mode_from_guest = |guest: AddressMode| -> wit::client_sampler::AddressMode {
        match guest {
            AddressMode::ClampToEdge => wit::client_sampler::AddressMode::ClampToEdge,
            AddressMode::Repeat => wit::client_sampler::AddressMode::Repeat,
            AddressMode::MirrorRepeat => wit::client_sampler::AddressMode::MirrorRepeat,
        }
    };
    let filter_mode_from_guest = |guest: FilterMode| match guest {
        FilterMode::Nearest => wit::client_sampler::FilterMode::Nearest,
        FilterMode::Linear => wit::client_sampler::FilterMode::Linear,
    };

    let desc = wit::client_sampler::Descriptor {
        address_mode_u: address_mode_from_guest(desc.address_mode_u),
        address_mode_v: address_mode_from_guest(desc.address_mode_v),
        address_mode_w: address_mode_from_guest(desc.address_mode_w),
        mag_filter: filter_mode_from_guest(desc.mag_filter),
        min_filter: filter_mode_from_guest(desc.min_filter),
        mipmap_filter: filter_mode_from_guest(desc.mipmap_filter),
    };
    wit::client_sampler::create(desc)
}
