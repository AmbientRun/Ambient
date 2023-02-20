# Contributing

## API docs

To see the latest version of the API docs, run the following command in the `Ambient` repository:

```sh
cd guest/rust
cargo doc -p ambient_api --open --no-deps
```

## Releasing

1. Update workspace versions in `Cargo.toml`, `guest/rust/Cargo.toml` and `guest/rust/api/Cargo.toml` (ambient_api_macro version)
2. Run `cargo build` in the root directory and in `guest/rust`, so that `Cargo.lock` is created
2. Create a tag `v0.X.Y` and commit with the same name
3. Push to origin
