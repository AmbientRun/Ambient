# Contributing

## API docs

To see the latest version of the API docs, run the following command in the `Ambient` repository:

```sh
cd guest/rust
cargo doc -p ambient_api --open --no-deps
```

## Releasing

1. Run `cargo run --example main -p generate-docs` and `cargo run -- update-interface-components` to update the documentation from the codebase.
2. Update workspace versions in:
   - `Cargo.toml`
   - `docs/src/installing.md`
   - `guest/rust/Cargo.toml`
   - `guest/rust/api/Cargo.toml` (`ambient_api_macro` version).
3. Run `cargo build` in the root directory, and in `guest/rust`, so that `Cargo.lock` is created. The build process will update several files in the `guest` folder.
4. If on a UNIX system, run `./scripts/run_all_examples.sh` and visually verify that they work as expected.
5. Update the `CHANGELOG.md` at the root of the repository. Copy the unreleased block, set the version and date on the copy, and then empty out the unreleased block for the next release.
6. Ensure that `README.md` and `docs/src/introduction.md` match.
7. Make a commit with the above changes, and create a tag `v0.X.Y`.
8. Push to origin.
