#!/bin/sh
# Helper script to cut a release for both the host and the guest.
set -e

BASEDIR=$(dirname $(dirname $(realpath "$0")))
cd $BASEDIR
cargo release --no-publish --execute --no-push --no-tag "$1"
cd $BASEDIR/guest/rust
cargo release --no-publish --execute --tag-name "v{{version}}" "$1"