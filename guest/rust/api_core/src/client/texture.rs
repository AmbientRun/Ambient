use crate::global::ProceduralTextureHandle;
use crate::internal::conversion::*;
use crate::internal::wit;

#[derive(Clone, Copy)]
pub enum Format {
    Rgba8Unorm,
}

#[derive(Clone)]
pub struct Descriptor2d<'a> {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    pub data: &'a [u8],
}

pub fn create_2d(desc: &Descriptor2d) -> ProceduralTextureHandle {
    use wit::client_texture::Format as WitFormat;
    let format = match desc.format {
        Format::Rgba8Unorm => WitFormat::Rgba8Unorm,
    };

    wit::client_texture::create2d(wit::client_texture::Descriptor2d {
        width: desc.width,
        height: desc.height,
        format,
        data: desc.data,
    })
    .from_bindgen()
}

pub fn destroy(handle: ProceduralTextureHandle) {
    wit::client_texture::destroy(handle.into_bindgen());
}
