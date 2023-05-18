use crate::global::ProceduralTextureHandle;
use crate::internal::conversion::*;
use crate::internal::wit;

#[derive(Clone, Copy)]
pub enum Format {
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,
    R16Uint,
    R16Sint,
    R16Unorm,
    R16Snorm,
    R16Float,
    Rg8Unorm,
    Rg8Snorm,
    Rg8Uint,
    Rg8Sint,
    R32Uint,
    R32Sint,
    R32Float,
    Rg16Uint,
    Rg16Sint,
    Rg16Unorm,
    Rg16Snorm,
    Rg16Float,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgb9e5Ufloat,
    Rgb10a2Unorm,
    Rg11b10Float,
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Unorm,
    Rgba16Snorm,
    Rgba16Float,
    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,
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
        Format::R8Unorm => WitFormat::R8Unorm,
        Format::R8Snorm => WitFormat::R8Snorm,
        Format::R8Uint => WitFormat::R8Uint,
        Format::R8Sint => WitFormat::R8Sint,
        Format::R16Uint => WitFormat::R16Uint,
        Format::R16Sint => WitFormat::R16Sint,
        Format::R16Unorm => WitFormat::R16Unorm,
        Format::R16Snorm => WitFormat::R16Snorm,
        Format::R16Float => WitFormat::R16Float,
        Format::Rg8Unorm => WitFormat::Rg8Unorm,
        Format::Rg8Snorm => WitFormat::Rg8Snorm,
        Format::Rg8Uint => WitFormat::Rg8Uint,
        Format::Rg8Sint => WitFormat::Rg8Sint,
        Format::R32Uint => WitFormat::R32Uint,
        Format::R32Sint => WitFormat::R32Sint,
        Format::R32Float => WitFormat::R32Float,
        Format::Rg16Uint => WitFormat::Rg16Uint,
        Format::Rg16Sint => WitFormat::Rg16Sint,
        Format::Rg16Unorm => WitFormat::Rg16Unorm,
        Format::Rg16Snorm => WitFormat::Rg16Snorm,
        Format::Rg16Float => WitFormat::Rg16Float,
        Format::Rgba8Unorm => WitFormat::Rgba8Unorm,
        Format::Rgba8UnormSrgb => WitFormat::Rgba8UnormSrgb,
        Format::Rgba8Snorm => WitFormat::Rgba8Snorm,
        Format::Rgba8Uint => WitFormat::Rgba8Uint,
        Format::Rgba8Sint => WitFormat::Rgba8Sint,
        Format::Bgra8Unorm => WitFormat::Bgra8Unorm,
        Format::Bgra8UnormSrgb => WitFormat::Bgra8UnormSrgb,
        Format::Rgb9e5Ufloat => WitFormat::Rgb9e5Ufloat,
        Format::Rgb10a2Unorm => WitFormat::Rgb10a2Unorm,
        Format::Rg11b10Float => WitFormat::Rg11b10Float,
        Format::Rg32Uint => WitFormat::Rg32Uint,
        Format::Rg32Sint => WitFormat::Rg32Sint,
        Format::Rg32Float => WitFormat::Rg32Float,
        Format::Rgba16Uint => WitFormat::Rgba16Uint,
        Format::Rgba16Sint => WitFormat::Rgba16Sint,
        Format::Rgba16Unorm => WitFormat::Rgba16Unorm,
        Format::Rgba16Snorm => WitFormat::Rgba16Snorm,
        Format::Rgba16Float => WitFormat::Rgba16Float,
        Format::Rgba32Uint => WitFormat::Rgba32Uint,
        Format::Rgba32Sint => WitFormat::Rgba32Sint,
        Format::Rgba32Float => WitFormat::Rgba32Float,
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
