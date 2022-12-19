mod assets;
mod error;
mod mixer;
// mod sink;
mod stream;

mod barycentric;
pub mod blt;
/// Fast fourier transform
pub mod hrtf;
pub mod signal;
pub mod source;
mod spatial;
pub mod track;
pub mod util;
pub mod value;
pub mod vorbis;
pub mod wav;

pub use assets::*;
pub use error::*;
pub use mixer::*;
// pub use sink::*;
pub use source::*;
pub use spatial::*;
pub use stream::*;

pub const MAX_CHANNELS: usize = 8;

pub type ChannelCount = u16;
pub type SampleRate = u64;
pub type Frame = glam::Vec2;
