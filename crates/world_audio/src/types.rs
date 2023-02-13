/// This file contains the type definitions for the audio library and is intended to be used by
/// both the client and the host.
///
/// It does **not** contain implementations that require dims internal functionality
use std::time::Duration;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Description to assemble an audio source graph
pub enum AudioNode {
    /// A source which does nothing
    Identity,
    /// Play from a vorbiss `.ogg` file from a url
    Vorbis {
        /// Url asset
        url: String,
    },
}
