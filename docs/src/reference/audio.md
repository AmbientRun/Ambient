# Audio

Ambient has basic audio functionality including sound playback, panning and volume control.

3D audio with HRTF is also included but considered as highly experimental.

## Usage

To use audio, you need to put the audio files into the `assets` folder, and then edit the `pipeline.toml`.

Check the `assets` folder in the [physics example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/physics) to see how this is done.

Audio should be loaded and played in clientside WASM/`client.rs` (the API is not supported on the server). [Messages](ember.md#messages--messages) can be used by the server to tell the client to play a sound effect.

# Examples with audio

- `./guest/rust/examples/basics/physics` (spatial audio)
- `./guest/rust/examples/basics/first_person_camera` (spatial audio)
- `./guest/rust/examples/games/music_sequencer`
- `./guest/rust/examples/ui/audio_ctrl`

The general idea is that in the ECS system, you can create an `audio::AudioPlayer` or `audio::SpatialAudioPlayer`. You can set the property of these players with methods such as `set_amplitude`. Then you can use the `player` to play a sound assets. This will actually return an `EntityId`. By `add_component` to the entity, you can control the playing sound as well. The `audio_ctrl` example shows the details.

```rust
pub fn main() {
    let player = audio::AudioPlayer::new();
    player.set_amplitude();
    let playing_sound = player.play(asset::url("assets/sound.ogg"));
    entity::add_component(s, amplitude(), 0.1);
}
```

## Deciding whether to convert audio formats

Currently, we support `wav`, `mp3`, and `ogg` audio file formats. If you use an `mp3` format, it will be converted to `ogg` during the build process. However, you can use either ".mp3" or ".ogg" in the `asset::url` function.

In some cases, you may want to explicitly control whether the audio is converted in order to save space or maintain the best audio quality. This is particularly relevant for `wav` files, which are large when unconverted but offer lossless playback. You can manage this setting in the `pipeline.toml` file.

```toml
[[pipelines]]
type = "Audio"
convert = true
```

If you convert a `wav` file, then you need to use `.ogg` in `asset::url`.
If the `convert` entry is missing, the default behaviour is no conversion.
