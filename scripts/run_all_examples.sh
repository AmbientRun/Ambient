#!/bin/sh
# Helper script to run all examples for quick testing. Could be replaced with something more fancy
# at a later stage.
set -e

BASEDIR=$(dirname $(dirname $(realpath "$0")))
PATHS=$(find "$BASEDIR/guest/rust/examples" -mindepth 1 -maxdepth 1 -type d | sort)
while IFS= read -r line; do
    cargo run -- --project-path "$line" run
done <<< "$PATHS"