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

impl IntoBindgen for Format {
    type Item = wit::client_texture::Format;

    fn into_bindgen(self) -> Self::Item {
        match self {
            Format::R8Unorm => Self::Item::R8Unorm,
            Format::R8Snorm => Self::Item::R8Snorm,
            Format::R8Uint => Self::Item::R8Uint,
            Format::R8Sint => Self::Item::R8Sint,
            Format::R16Uint => Self::Item::R16Uint,
            Format::R16Sint => Self::Item::R16Sint,
            Format::R16Unorm => Self::Item::R16Unorm,
            Format::R16Snorm => Self::Item::R16Snorm,
            Format::R16Float => Self::Item::R16Float,
            Format::Rg8Unorm => Self::Item::Rg8Unorm,
            Format::Rg8Snorm => Self::Item::Rg8Snorm,
            Format::Rg8Uint => Self::Item::Rg8Uint,
            Format::Rg8Sint => Self::Item::Rg8Sint,
            Format::R32Uint => Self::Item::R32Uint,
            Format::R32Sint => Self::Item::R32Sint,
            Format::R32Float => Self::Item::R32Float,
            Format::Rg16Uint => Self::Item::Rg16Uint,
            Format::Rg16Sint => Self::Item::Rg16Sint,
            Format::Rg16Unorm => Self::Item::Rg16Unorm,
            Format::Rg16Snorm => Self::Item::Rg16Snorm,
            Format::Rg16Float => Self::Item::Rg16Float,
            Format::Rgba8Unorm => Self::Item::Rgba8Unorm,
            Format::Rgba8UnormSrgb => Self::Item::Rgba8UnormSrgb,
            Format::Rgba8Snorm => Self::Item::Rgba8Snorm,
            Format::Rgba8Uint => Self::Item::Rgba8Uint,
            Format::Rgba8Sint => Self::Item::Rgba8Sint,
            Format::Bgra8Unorm => Self::Item::Bgra8Unorm,
            Format::Bgra8UnormSrgb => Self::Item::Bgra8UnormSrgb,
            Format::Rgb9e5Ufloat => Self::Item::Rgb9e5Ufloat,
            Format::Rgb10a2Unorm => Self::Item::Rgb10a2Unorm,
            Format::Rg11b10Float => Self::Item::Rg11b10Float,
            Format::Rg32Uint => Self::Item::Rg32Uint,
            Format::Rg32Sint => Self::Item::Rg32Sint,
            Format::Rg32Float => Self::Item::Rg32Float,
            Format::Rgba16Uint => Self::Item::Rgba16Uint,
            Format::Rgba16Sint => Self::Item::Rgba16Sint,
            Format::Rgba16Unorm => Self::Item::Rgba16Unorm,
            Format::Rgba16Snorm => Self::Item::Rgba16Snorm,
            Format::Rgba16Float => Self::Item::Rgba16Float,
            Format::Rgba32Uint => Self::Item::Rgba32Uint,
            Format::Rgba32Sint => Self::Item::Rgba32Sint,
            Format::Rgba32Float => Self::Item::Rgba32Float,
        }
    }
}

#[derive(Clone)]
pub struct Descriptor2D<'a> {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    pub data: &'a [u8],
}

impl<'a> IntoBindgen for Descriptor2D<'a> {
    type Item = wit::client_texture::Descriptor2d<'a>;

    fn into_bindgen(self) -> Self::Item {
        Self::Item {
            width: self.width,
            height: self.height,
            format: self.format.into_bindgen(),
            data: self.data,
        }
    }
}

pub fn create_2d(desc: &Descriptor2D) -> ProceduralTextureHandle {
    wit::client_texture::create2d(desc.clone().into_bindgen()).from_bindgen()
}

pub fn destroy(handle: ProceduralTextureHandle) {
    wit::client_texture::destroy(handle.into_bindgen());
}
