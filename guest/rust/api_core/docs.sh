#!/bin/sh
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc -p ambient_api --all-features $@