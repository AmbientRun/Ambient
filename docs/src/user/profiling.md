# Profiling

Ambient supports profiling through [puffin](https://github.com/EmbarkStudios/puffin). To use it, follow these steps;

1. Build Ambient with profiling enabled (add th `profile` feature flag); `cargo install --path app --features profile` from the root folder.
2. Install (puffin viewer](https://crates.io/crates/puffin_viewer): `cargo install puffin_viewer`
3. Start ambient: `ambient run guest/examples/basics/primitives`
4. Start puffin viewer; `puffin_viewer`

You should now see real-time performance metrics for Ambient.
