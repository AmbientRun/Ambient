# Profiling

Ambient supports profiling through [puffin](https://github.com/EmbarkStudios/puffin). To use it, follow these steps:

1. Build Ambient with profiling enabled (add the `profile` feature flag). From the root folder:

   ```sh
   cargo install --path app --features profile
   ```

2. Install [puffin_viewer](https://crates.io/crates/puffin_viewer):

   ```sh
   cargo install puffin_viewer
   ```

3. Start Ambient:

   ```sh
   ambient run guest/examples/basics/primitives
   ```

4. Start `puffin_viewer`:

   ```sh
   puffin_viewer
   ```

You should now see real-time performance metrics for Ambient.
