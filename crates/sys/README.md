# Ambient Sys

Platform agnostic abstractions for Ambient.

The intent of this crate is to provide a common runtime and platform agnostic interface for WebAssembly and native targets, e.g; timers and sleep, file io, task spawning, etc.

The crate uses `tokio` for native platform, and uses the web/js api on WebAssembly.
