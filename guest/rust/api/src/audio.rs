use crate::internal::wit;

/// Load an audio file from `url`, and return an [AudioTrack] that can be used to play the audio.
pub fn load(url: String) -> AudioTrack {
    wit::audio::load(&url);
    AudioTrack {
        name: url,
        looping: false,
        volume: 1.0,
    }
}

/// Represents an audio track that can be played.
#[derive(Clone, Debug)]
pub struct AudioTrack {
    /// The name of the audio
    pub name: String,
    /// Whether or not the audio should loop
    pub looping: bool,
    /// The volume of the audio
    pub volume: f32,
}

impl AudioTrack {
    /// Set whether or not the track should loop.
    pub fn looping(&self, looping: bool) -> Self {
        Self {
            looping,
            ..self.clone()
        }
    }

    /// Set the volume of the track.
    pub fn volume(&self, volume: f32) -> Self {
        Self {
            volume,
            ..self.clone()
        }
    }

    /// Play the track.
    pub fn play(&self) {
        wit::audio::play(&self.name, self.looping, self.volume);
    }
}
