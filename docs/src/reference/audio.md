# Audio

Ambient now has some very basic audio functions including sound playback, looping and volome control.

## Usage

To use audio, you need to put the audio files into the `assets` folder, and then edit the `pipeline.json`.

Check the [physics example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/physics), especially the `assets` folder to see how this is done.

Audio should be loaded and played in `client.rs`. Thus, `messages` can be useful to trigger sound effect in some cases.

> Hint: for messages, you need to edit the `ambient.toml`.

## Caveat

So far we support `wav`, `mp3` and `ogg`, but if you use `wav`/`mp3`, it will be converted to `ogg` and you should use `ogg` in your code inside the client main function.

```rust
// "src/client.rs" in the "pong" example
#[main]
pub fn main() {
    // if your audio file is "bgm.wav", you still need to use "ogg" here
    let bgm = audio::load(asset::url("assets/bgm.ogg").unwrap());
    bgm.looping(true).scale(0.2).play();
}
```