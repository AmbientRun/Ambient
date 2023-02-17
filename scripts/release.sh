#!/bin/sh
# Helper script to cut a release for both the host and the guest.
set -e

BASEDIR=$(dirname $(dirname $(realpath "$0")))
cd $BASEDIR
cargo release --no-publish --no-push "$1"
cd $BASEDIR/guest/rust
cargo release --no-publish --no-push "$1"