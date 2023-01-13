use std::path::Path;

use gltf::{buffer, image::Format, Document, Gltf};
use image::{
    DynamicImage, ImageFormat::{Jpeg, Png}
};

pub struct GltfImport {
    pub name: String,
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}
impl GltfImport {
    pub fn from_slice<S: AsRef<[u8]>>(name: String, import_images: bool, slice: S) -> gltf::Result<Self> {
        let Gltf { document, blob } = Gltf::from_slice(slice.as_ref())?;
        let buffers = import_buffer_data(&document, None, blob)?;
        let images = if import_images { import_image_data(&document, None, &buffers)? } else { Vec::new() };
        Ok(Self { name, document, buffers, images })
    }
}

// All of the below is basically just copied from the gltf crate, except it doesn't panic on bad resource references

fn import_buffer_data(document: &Document, base: Option<&Path>, mut blob: Option<Vec<u8>>) -> gltf::Result<Vec<buffer::Data>> {
    let mut buffers = Vec::new();
    for buffer in document.buffers() {
        let mut data = match buffer.source() {
            buffer::Source::Uri(uri) if base.is_some() => Scheme::read(base.unwrap(), uri),
            buffer::Source::Bin => blob.take().ok_or(gltf::Error::MissingBlob),
            _ => Ok(Vec::new()),
        }?;
        // if data.len() < buffer.length() {
        //     return Err(
        //         gltf::Error::BufferLength {
        //             buffer: buffer.index(),
        //             expected: buffer.length(),
        //             actual: data.len(),
        //         }
        //     );
        // }
        while data.len() % 4 != 0 {
            data.push(0);
        }
        buffers.push(buffer::Data(data));
    }
    Ok(buffers)
}

fn import_image_data(document: &Document, base: Option<&Path>, buffer_data: &[buffer::Data]) -> gltf::Result<Vec<gltf::image::Data>> {
    let mut images = Vec::new();
    #[cfg(feature = "guess_mime_type")]
    let guess_format = |encoded_image: &[u8]| match image_crate::guess_format(encoded_image) {
        Ok(image_crate::ImageFormat::Png) => Some(Png),
        Ok(image_crate::ImageFormat::Jpeg) => Some(Jpeg),
        _ => None,
    };
    #[cfg(not(feature = "guess_mime_type"))]
    let guess_format = |_encoded_image: &[u8]| None;
    for image in document.images() {
        match image.source() {
            gltf::image::Source::Uri { uri, mime_type } if base.is_some() => {
                match Scheme::parse(uri) {
                    Scheme::Data(Some(annoying_case), base64) => {
                        let encoded_image = base64::decode(base64).map_err(gltf::Error::Base64)?;
                        let encoded_format = match annoying_case {
                            "image/png" => Png,
                            "image/jpeg" => Jpeg,
                            _ => match guess_format(&encoded_image) {
                                Some(format) => format,
                                None => return Err(gltf::Error::UnsupportedImageEncoding),
                            },
                        };
                        let decoded_image = image::load_from_memory_with_format(&encoded_image, encoded_format)?;
                        images.push(new_image_data(decoded_image));
                        continue;
                    }
                    Scheme::Unsupported => return Err(gltf::Error::UnsupportedScheme),
                    _ => {}
                }
                let encoded_image = Scheme::read(base.unwrap(), uri)?;
                let encoded_format = match mime_type {
                    Some("image/png") => Png,
                    Some("image/jpeg") => Jpeg,
                    Some(_) => match guess_format(&encoded_image) {
                        Some(format) => format,
                        None => return Err(gltf::Error::UnsupportedImageEncoding),
                    },
                    None => match uri.rsplit('.').next() {
                        Some("png") => Png,
                        Some("jpg") | Some("jpeg") => Jpeg,
                        _ => match guess_format(&encoded_image) {
                            Some(format) => format,
                            None => return Err(gltf::Error::UnsupportedImageEncoding),
                        },
                    },
                };
                let decoded_image = image::load_from_memory_with_format(&encoded_image, encoded_format)?;
                images.push(new_image_data(decoded_image));
            }
            gltf::image::Source::View { view, mime_type } => {
                let parent_buffer_data = &buffer_data[view.buffer().index()].0;
                let begin = view.offset();
                let end = begin + view.length();
                let encoded_image = &parent_buffer_data[begin..end];
                let encoded_format = match mime_type {
                    "image/png" => Png,
                    "image/jpeg" => Jpeg,
                    _ => match guess_format(encoded_image) {
                        Some(format) => format,
                        None => return Err(gltf::Error::UnsupportedImageEncoding),
                    },
                };
                let decoded_image = image::load_from_memory_with_format(encoded_image, encoded_format)?;
                images.push(new_image_data(decoded_image));
            }
            _ => {
                images.push(gltf::image::Data { format: Format::R8G8B8A8, width: 1, height: 1, pixels: vec![255, 255, 255, 255] });
            }
        }
    }

    Ok(images)
}

fn new_image_data(image: DynamicImage) -> gltf::image::Data {
    use image::GenericImageView;
    let format = match image {
        DynamicImage::ImageLuma8(_) => Format::R8,
        DynamicImage::ImageLumaA8(_) => Format::R8G8,
        DynamicImage::ImageRgb8(_) => Format::R8G8B8,
        DynamicImage::ImageRgba8(_) => Format::R8G8B8A8,
        DynamicImage::ImageLuma16(_) => Format::R16,
        DynamicImage::ImageLumaA16(_) => Format::R16G16,
        DynamicImage::ImageRgb16(_) => Format::R16G16B16,
        DynamicImage::ImageRgba16(_) => Format::R16G16B16A16,
        DynamicImage::ImageRgb32F(_) => Format::R32G32B32FLOAT,
        DynamicImage::ImageRgba32F(_) => Format::R32G32B32A32FLOAT,
        _ => todo!(),
    };
    let (width, height) = image.dimensions();
    let pixels = image.into_bytes();
    gltf::image::Data { format, width, height, pixels }
}

/// Represents the set of URI schemes the importer supports.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Scheme<'a> {
    /// `data:[<media type>];base64,<data>`.
    Data(Option<&'a str>, &'a str),

    /// `file:[//]<absolute file path>`.
    ///
    /// Note: The file scheme does not implement authority.
    File(&'a str),

    /// `../foo`, etc.
    Relative,

    /// Placeholder for an unsupported URI scheme identifier.
    Unsupported,
}

impl Scheme<'_> {
    fn parse(uri: &str) -> Scheme {
        if uri.contains(':') {
            if let Some(postfix) = uri.strip_prefix("data:") {
                let match0 = &postfix.split(";base64,").next();
                let match1 = &postfix.split(";base64,").nth(1);
                if match1.is_some() {
                    Scheme::Data(Some(match0.unwrap()), match1.unwrap())
                } else if match0.is_some() {
                    Scheme::Data(None, match0.unwrap())
                } else {
                    Scheme::Unsupported
                }
            } else if let Some(postfix) = uri.strip_prefix("file://") {
                Scheme::File(postfix)
            } else if let Some(postfix) = uri.strip_prefix("file:") {
                Scheme::File(postfix)
            } else {
                Scheme::Unsupported
            }
        } else {
            Scheme::Relative
        }
    }

    fn read(base: &Path, uri: &str) -> gltf::Result<Vec<u8>> {
        match Scheme::parse(uri) {
            Scheme::Data(_, base64) => base64::decode(base64).map_err(gltf::Error::Base64),
            Scheme::File(path) => read_to_end(path),
            Scheme::Relative => read_to_end(base.join(uri)),
            Scheme::Unsupported => Err(gltf::Error::UnsupportedScheme),
        }
    }
}

fn read_to_end<P>(path: P) -> gltf::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    use std::io::Read;
    let file = std::fs::File::open(path.as_ref()).map_err(gltf::Error::Io)?;
    // Allocate one extra byte so the buffer doesn't need to grow before the
    // final `read` call at the end of the file.  Don't worry about `usize`
    // overflow because reading will fail regardless in that case.
    let length = file.metadata().map(|x| x.len() + 1).unwrap_or(0);
    let mut reader = std::io::BufReader::new(file);
    let mut data = Vec::with_capacity(length as usize);
    reader.read_to_end(&mut data).map_err(gltf::Error::Io)?;
    Ok(data)
}
