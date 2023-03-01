#!/bin/sh
set -e

rm -rf tmp/examples
mkdir -p tmp
cp -r guest/rust/examples tmp/examples

NEW_RUST_VERSION=`grep '^rust-version = ' Cargo.toml | tr -d '\n'`
sed -i "s/rust-version = { workspace = true }/$NEW_RUST_VERSION/" tmp/examples/*/Cargo.toml

NEW_VERSION=`grep '^version = ' Cargo.toml | tr -d '\n'`
sed -i "s/version = { workspace = true }/$NEW_VERSION/" tmp/examples/*/Cargo.toml

sed -i "s/path = \"..\/..\/api\"/$NEW_VERSION/" tmp/examples/*/Cargo.toml

cd tmp
zip -r -q examples.zip examples
rm -rf examples