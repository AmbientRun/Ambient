use crate::global::ProceduralTextureHandle;
use crate::internal::conversion::*;
use crate::internal::wit;

#[derive(Clone, Copy)]
pub enum Format {
    Rgba8Unorm,
}

pub fn create_2d(width: u32, height: u32, format: Format, data: &[u8]) -> ProceduralTextureHandle {
    use wit::client_texture::Format as WitFormat;
    let format = match format {
        Format::Rgba8Unorm => WitFormat::Rgba8Unorm,
    };
    wit::client_texture::create2d(width, height, format, data).from_bindgen()
}
