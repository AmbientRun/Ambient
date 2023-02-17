#!/bin/sh
# Helper script to cut a release for both the host and the guest.
#
# At time of writing, this does not publish to crates.io for either host or guest.
#  - Host: blocked on dependencies
#  - Guest: blocked on API finalization
set -e

BASEDIR=$(dirname $(dirname $(realpath "$0")))

cd $BASEDIR/guest/rust
cargo release version --no-confirm --quiet --execute "$1"
cargo release replace --no-confirm --quiet --execute

cd $BASEDIR
cargo release version --no-confirm --quiet --execute "$1"
cargo release commit --no-confirm --quiet --execute
cargo release tag --no-confirm --quiet --execute --tag-name "v{{version}}"
cargo release push --no-confirm --quiet --execute