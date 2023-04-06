#!/bin/sh
set -e

export RUSTFLAGS="-D warnings"

cargo clippy
cargo clippy --features client
cargo clippy --features server
cargo clippy --features client,server