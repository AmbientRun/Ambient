# Shared crates

Crates that are shared between the host and the Rust guest.

They should only depend on `crates.io`, `libs` or each other. Dependencies to `crates` are allowed if it's for native-only code or for dev dependencies.
