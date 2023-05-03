# Audio

Ambient has basic audio functionality including sound playback, looping and volume control.

## Usage

To use audio, you need to put the audio files into the `assets` folder, and then edit the `pipeline.json`.

Check the `assets` folder in the [physics example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/physics) to see how this is done.

Audio should be loaded and played in clientside WASM/`client.rs` (the API is not supported on the server). [Messages](project.md#messages--messages) can be used by the server to tell the client to play a sound effect.

# Examples with audio

- `./guest/rust/examples/basics/physics`
- `./guest/rust/examples/games/pong`
- `./guest/rust/examples/games/minigolf`
- `./guest/rust/examples/games/music_sequencer`

## Deciding whether to convert audio formats

Currently, we support `wav`, `mp3`, and `ogg` audio file formats. If you use an `mp3` format, it will be converted to `ogg` during the build process. However, you can use either ".mp3" or ".ogg" in the `audio::load` function.

```rust
#[main]
pub fn main() {
    // if your audio file is "bgm.mp3", you can use either "mp3" or "ogg" here
    let bgm = audio::load(asset::url("assets/bgm.ogg").unwrap());
    bgm.looping(true).scale(0.2).play();
}
```

In some cases, you may want to explicitly control whether the audio is converted in order to save space or maintain the best audio quality. This is particularly relevant for `wav` files, which are large when unconverted but offer lossless playback. You can manage this setting in the `pipeline.json` file.

```json
{
    "pipeline": {
        "type": "Audio",
        "convert": false
    }
}
```

If you convert a `wav` file, then you need to use ".ogg" in `audio::load`.
If the `convert` entry is missing, the default behaviour is no convertion.