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
    pub fn looping(&mut self, looping: bool) -> &mut Self {
        self.looping = looping;
        self
    }

    /// Set the volume of the audio
    pub fn scale(&mut self, amp: f32) -> &mut Self {
        self.amp = amp;
        self
    }

    /// Play the audio
    pub fn play(&mut self) {
        wit::audio::play(&self.name, self.looping, self.amp);
    }
}