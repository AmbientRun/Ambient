# Development

## Documentation

To see the documentation as it will appear on docs.rs, use

```sh
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc -p ambient_api --all-features --open
```

or `./docs.sh --open`.

The "feature boxes" are a nightly-only feature, but docs.rs builds with nightly.
This command mirrors the configuration that docs.rs uses.
