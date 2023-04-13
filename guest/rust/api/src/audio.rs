use crate::internal::wit;

/// Add sound to the audio library in the world you call this
pub fn add_sound(name: &str, url: String) {
    wit::audio::add_track(name, &url);
}

/// Play audio
pub fn get(name: &str) -> AudioQuery {
    AudioQuery {
        name: name.to_string(),
        looping: false,
        amp: 1.0,
    }
}

/// The audio query, used to play audio
pub struct AudioQuery {
    /// The name of the audio
    pub name: String,
    /// Whether or not the audio should loop
    pub looping: bool,
    /// The volume of the audio
    pub amp: f32,
}

impl AudioQuery {

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