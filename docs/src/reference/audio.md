# Audio

Ambient has basic audio functionality including sound playback, looping and volume control.

## Usage

To use audio, you need to put the audio files into the `assets` folder, and then edit the `pipeline.json`.

Check the `assets` folder in the [physics example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/physics) to see how this is done.

Audio should be loaded and played in clientside WASM/`client.rs` (the API is not supported on the server). [Messages](project.md#messages--messages) can be used by the server to tell the client to play a sound effect.

## Caveat

At present, we support `wav`, `mp3` and `ogg` through `ffmpeg`. Note that if a non-`ogg` format is used, it will be converted to `ogg`.


You can decide whether to use the `ogg` extension in your code or keep the original one. Both will work.

```rust
// "src/client.rs" in the "pong" example
#[main]
pub fn main() {
    // if your audio file is "bgm.wav", you can use "ogg" here
    let bgm = audio::load(asset::url("assets/bgm.ogg").unwrap());
    // the alternative is also fine
    // let bgm = audio::load(asset::url("assets/bgm.wav").unwrap());
    bgm.looping(true).scale(0.2).play();
}
```
