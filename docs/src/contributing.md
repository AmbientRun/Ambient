# Contributing

## API docs

To see the latest version of the API docs, run the following command in the `Ambient` repository:

```sh
cd guest/rust
cargo doc -p ambient_api --open --no-deps
```

## Releasing

1. Update workspace versions in:
   - `README.md`
   - `Cargo.toml`
   - `docs/src/installing.md`
   - `guest/rust/Cargo.toml`
   - `guest/rust/api/Cargo.toml` (`ambient_api_macro` version).
2. Run `cargo build` in the root directory, and in `guest/rust`, so that `Cargo.lock` is created. The build process will update several files in the `guest` folder.
3. Make a commit with the above changes, and create a tag `v0.X.Y`.
4. Push to origin.
