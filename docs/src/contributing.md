# Contributing

## API docs

To see the latest version of the API docs, run the following command in the `Ambient` repository:

```sh
cd guest/rust
cargo doc -p ambient_api --open --no-deps
```

## Releasing

1. Update workspace versions in `Cargo.toml` and `guest/rust/Cargo.toml`
2. Create a tag `v0.0.7` and commit with the same name
3. Push to origin
