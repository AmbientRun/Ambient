use crate::internal::wit;

/// Add sound to the audio library in the world you call this
pub fn load(url: String) -> AudioTrack {
    wit::audio::load(&url);
    AudioTrack {
        name: url,
        looping: false,
        amp: 1.0,
    }
}

/// The audio query, used to play audio
#[derive(Clone, Debug)]
pub struct AudioTrack {
    /// The name of the audio
    pub name: String,
    /// Whether or not the audio should loop
    pub looping: bool,
    /// The volume of the audio
    pub amp: f32,
}

impl AudioTrack {
    /// Set whether or not the audio should loop
    pub fn looping(&self, looping: bool) -> Self {
        Self {
            looping,
            ..self.clone()
        }
    }

    /// Set the volume of the audio
    pub fn scale(&self, amp: f32) -> Self {
        Self {
            amp,
            ..self.clone()
        }
    }

    /// Play the audio
    pub fn play(&self) {
        wit::audio::play(&self.name, self.looping, self.amp);
    }
}
