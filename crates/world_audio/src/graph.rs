use std::{fmt::Debug, str::FromStr};

use ambient_audio::{Source, VorbisFromUrl};
use ambient_native_std::{
    self,
    asset_cache::{AssetCache, AsyncAssetKeyExt},
    asset_url::AbsAssetUrl,
};
use rand::{thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

use crate::error::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Textual representation of a node in the audio graph which specifies how to construct a Sound.
pub enum AudioNode {
    /// A source which does nothing
    Identity,
    /// Play from a vorbis `.ogg` file from a url
    Vorbis {
        /// Url asset
        url: String,
    },
}

impl Default for AudioNode {
    fn default() -> Self {
        Self::Identity
    }
}

impl AudioNode {
    /// Builds the adapter into a proper source.
    /// If the graph can not immediately be built, it returns None
    pub fn try_build(
        self,
        assets: &AssetCache,
        _seed: AudioSeed,
    ) -> Result<Option<Box<dyn Source>>> {
        match self {
            AudioNode::Vorbis { url } => {
                let track = VorbisFromUrl {
                    url: AbsAssetUrl::from_str(&url).unwrap(),
                }
                .peek(assets)
                .transpose()?;
                match track {
                    Some(track) => Ok(Some(Box::new(track.decode()))),
                    None => Ok(None),
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
/// The seed for deterministic random state for replicating a sound effect on many clients
pub struct AudioSeed {
    pub rng_seed: <ChaCha12Rng as SeedableRng>::Seed,
}

impl AudioSeed {
    pub fn new() -> Self {
        Self {
            rng_seed: thread_rng().gen(),
        }
    }
}

impl Default for AudioSeed {
    fn default() -> Self {
        Self::new()
    }
}
