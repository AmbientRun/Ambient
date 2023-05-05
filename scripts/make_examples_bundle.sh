#!/bin/sh
set -e

rm -rf tmp/examples
mkdir -p tmp
cp -r guest/rust/examples tmp/examples

NEW_RUST_VERSION=`grep '^rust-version = ' Cargo.toml | tr -d '\n'`
sed -i -E "s/rust-version = \{ ?workspace = true ?\}/$NEW_RUST_VERSION/" tmp/examples/*/*/Cargo.toml

NEW_VERSION=`grep '^version = ' Cargo.toml | tr -d '\n'`
sed -i -E "s/version = \{ ?workspace = true ?\}/$NEW_VERSION/" tmp/examples/*/*/Cargo.toml

sed -i -E "s/ambient_([a-z_]+) ?= ?.*/ambient_\1 = \{ $NEW_VERSION \}/" tmp/examples/*/*/Cargo.toml

cd tmp
rm -rf examples.zip
zip -r -q examples.zip examples
rm -rf examples
