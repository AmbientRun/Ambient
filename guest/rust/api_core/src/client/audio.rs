use crate::internal::wit;

/// Load an audio file from `url`, and return an [AudioTrack] that can be used to play the audio.
pub fn load(url: String) -> AudioTrack {
    let actuall_url = url.replace(".mp3", ".ogg");
    wit::client_audio::load(&actuall_url);
    AudioTrack {
        name: actuall_url,
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
    pub fn looping(&mut self, looping: bool) -> &mut Self {
        self.looping = looping;
        self
    }

    /// Set the volume of the track.
    pub fn volume(&mut self, volume: f32) -> &mut Self {
        self.volume = volume;
        wit::client_audio::set_volume(&self.name, volume);
        self
    }

    /// Play the track.
    pub fn play(&self) -> AudioTrackId {
        let uid = rand::random::<u32>();
        wit::client_audio::play(&self.name, self.looping, self.volume, uid);
        AudioTrackId { uid }
    }

    /// Stop the track.
    pub fn stop(&self) {
        wit::client_audio::stop(&self.name);
    }
}

/// Audio tracks are identified by an [AudioTrackId].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioTrackId {
    /// The unique id of the audio track.
    pub uid: u32,
}

impl AudioTrackId {
    /// Stop the track.
    pub fn stop(&self) {
        wit::client_audio::stop_by_id(self.uid);
    }
}
