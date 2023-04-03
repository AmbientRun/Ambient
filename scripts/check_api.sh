#!/bin/sh
set -e

RUSTFLAGS="-D warnings"

cd guest/rust
cargo check
cargo check --features client
cargo check --features server
cargo check --features client,server