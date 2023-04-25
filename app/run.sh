#!/bin/bash

if [ $# -ne 1 ]; then
    echo "Usage: ./run.sh <example_name>"
    exit 1
fi

example_name="$1"
example_path="../guest/rust/examples/${example_name}"
cargo run -r -- run "${example_path}"