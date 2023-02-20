# Installing

A [Rust](https://www.rust-lang.org/) toolchain is required to install Ambient. Once you have a toolchain available, run the following command to install the Ambient executable into your `PATH` (assuming `cargo` is in your `PATH`):

```sh
cargo install --git https://github.com/AmbientOrg/Ambient.git --tag v0.0.14 ambient
```

## Dependencies: Linux/Ubuntu

These dependencies are required to build the C++ components of Ambient (i.e. PhysX). Please let us know if additional dependencies are required on your system.

```sh
apt-get install -y build-essential cmake pkg-config libfontconfig1-dev clang libasound2-dev ninja-build
```

## Running on headless Linux/Ubuntu

Install the above dependencies, then install the following:

```sh
add-apt-repository ppa:oibaf/graphics-drivers -y
apt-get update
apt install -y libxcb-xfixes0-dev mesa-vulkan-drivers
```

Ambient currently assumes that you have access to GPU drivers (but not necessarily a _GPU_ by itself) in headless mode. This requirement may be relaxed in future.
