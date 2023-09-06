# Basic Dockerfile to build and run the server in a Linux environment.
# The official Rust base Docker image uses Debian.
FROM rust:1.70.0
WORKDIR /app

RUN apt-get update && \
    apt-get install -y \
    zip build-essential cmake pkg-config \
    libfontconfig1-dev clang libasound2-dev ninja-build \
    libxcb-xfixes0-dev mesa-vulkan-drivers