#!/bin/sh
set -e

export RUSTFLAGS="-D warnings"

cargo check
cargo check --features client
cargo check --features server
cargo check --features client,server