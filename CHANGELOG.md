# Changelog

<!-- markdownlint-disable-file MD024 -->

This changelog is manually updated. While an effort will be made to keep the [Unreleased](#unreleased-yyyy-mm-dd) changes up to date, it may not be fully representative of the current state of the project.

<!-- If you are updating this file, make sure you copy the unreleased section and change the version and date. Do not remove it. -->

## Unreleased (YYYY-MM-DD)

### Added

- Support for kinematic bodies. This is used by the minigolf example to provide its moving obstacles.
- Added `physics::move_character` function to correctly move character controllers. This is used by the third-person camera example.
- The client's window title is now automatically changed to the name of the project running on the server. Thanks to [@MavethGH](https://github.com/MavethGH) for implementing this in [#178](https://github.com/AmbientRun/Ambient/pull/178)!

### Changed

- Ambient is now dual-licensed MIT/Apache2, in accordance with the rest of the Rust ecosystem.
- [Trace target directory content for cache troubleshooting](https://github.com/AmbientRun/Ambient/pull/170), by [@daniellavoie](https://github.com/daniellavoie)

### Fixed

- Added attributions for external code.
- Typo fixes. Thanks to the following!
  - [#159](https://github.com/AmbientRun/Ambient/pull/159) by [@daniellavoie](https://github.com/daniellavoie)
  - [#172](https://github.com/AmbientRun/Ambient/pull/172) by [@eltociear](https://github.com/eltociear)
- The Minigolf example now moves its camera when right-click is held down to make it easier to play.

<!-- ### Removed -->

## Version 0.1.1 (2023-02-22)

### Added

- A [minigolf example](guest/rust/examples/minigolf) by [SK83RJOSH](https://github.com/SK83RJOSH).
- Examples are now bundled into a downloadable `examples.zip` for each release.

### Fixed

- macOS ARM64 builds are now available after enabling the execution of unsigned executable memory (as required for wasmtime execution).
- The debugging configuration for VSCode was updated to use the new CLI.
- Minor documentation updates.

## Version 0.1.0 (2023-02-22)

Initial release. See the [announcement blog post](https://www.ambient.run/post/introducing-ambient).
