use crate::internal::wit;
use crate::{
    components::core::{
        app::name,
        audio::{amplitude, audio_player, audio_url, looping, panning, trigger_at_this_frame},
    },
    entity,
    prelude::{Entity, EntityId},
};

// /// An audio track that can be played.
// pub struct AudioFile {
//     url: String,
// };

/// TODO: make this player seperate from the audio track
#[derive(Debug, Clone)]
pub struct AudioPlayer {
    /// The url of the audio file to play
    pub url: String,
    /// The entity that represents the audio player
    entity: EntityId,
}

impl AudioPlayer {
    /// Create new audio player from URL
    pub fn from_url(url: String) -> Self {
        let player = Entity::new()
            .with_default(audio_player())
            .with(name(), "Audio player".to_string())
            .with(audio_url(), url.clone())
            .with(trigger_at_this_frame(), false)
            .with(looping(), false)
            .with(amplitude(), 1.0)
            .with(panning(), 0.0)
            .spawn();
        Self {
            url,
            entity: player,
        }
    }
    /// set the looping of the sound
    pub fn set_looping(&self, val: bool) {
        entity::set_component(self.entity, looping(), val);
    }

    /// set the amp/volume of the sound
    pub fn set_amplitude(&self, amp: f32) {
        entity::set_component(self.entity, amplitude(), amp);
    }
    /// set the panning of the sound -1.0 is 100% left, 1.0 is 100% right.
    pub fn set_panning(&self, pan: f32) {
        entity::set_component(self.entity, panning(), pan);
    }
    /// play the sound: in ECS, the sound entity is sole,
    /// but with each play, there will be an uid returned
    /// incase you want to stop the sound with uid at some point
    pub fn play(&self) {
        entity::set_component(self.entity, trigger_at_this_frame(), true);
    }
}

/// Load an audio file from `url`, and return an [AudioTrack] that can be used to play the audio.
pub fn load(url: String) -> AudioTrack {
    let actuall_url = url.replace(".mp3", ".ogg");
    wit::client_audio::load(&actuall_url);
    AudioTrack {
        name: actuall_url,
        looping: false,
        volume: 1.0,
        fx: vec![],
    }
}

/// Audio effects enum, should be added to `AudioTrack` to apply them.
#[derive(Clone, Debug)]
pub enum AudioEffect {
    /// -1.0 is 100% left, 1.0 is 100% right.
    Panning(f32),
    /// Low pass filter, first params is frequency in Hz. the second is bandwidth
    Lpf(f32, f32),
    /// High pass filter, first params is frequency in Hz. the second is bandwidth
    Hpf(f32, f32),
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
    /// Effect like panning, filter, reverb, etc.
    pub fx: Vec<AudioEffect>,
}

impl AudioTrack {
    /// Set whether or not the track should loop.
    pub fn looping(&mut self, looping: bool) -> &mut Self {
        self.looping = looping;
        self
    }

    /// Set the volume of the track.
    pub fn volume(&mut self, volume: f32) -> &mut Self {
        self.volume = volume.max(0.);
        wit::client_audio::set_volume(&self.name, volume);
        self
    }

    /// Play the track.
    pub fn play(&self) -> AudioTrackId {
        let uid = rand::random::<u32>();
        // for fx in self.fx {
        //     match fx {
        //         AudioEffect::Panning(panning) => {
        //             wit::client_audio::set_fx(&self.name, "panning", vec![panning]);
        //         }
        //         AudioEffect::Lpf(frequency, bandwidth) => {
        //             wit::client_audio::set_fx(&self.name, "lpf", vec![frequency, bandwidth]);
        //         }
        //         AudioEffect::Hpf(frequency, bandwidth) => {
        //             wit::client_audio::set_fx(&self.name, "hpf", vec![frequency, bandwidth]);
        //         }
        //     }
        // }
        println!("play audio track: {}", self.name);
        wit::client_audio::play(&self.name, self.looping, self.volume, uid);
        AudioTrackId { uid }
    }

    /// Stop the track.
    pub fn stop(&self) {
        wit::client_audio::stop(&self.name);
    }

    /// Add panning effect
    pub fn pan(&mut self, panning: f32) -> &mut Self {
        self.fx.push(AudioEffect::Panning(panning));
        self
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
