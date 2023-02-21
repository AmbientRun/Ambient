# Installing

The easiest way to install Ambient is to simple download the latest binary release from the [GitHub releases](https://github.com/AmbientRun/Ambient/releases).

You will also need [Rust](https://www.rust-lang.org/) installed so that you can compile Ambient modules.

## Installing from the git repository

To install the latest version from the `main` branch, run the following:

```sh
cargo install --git https://github.com/AmbientRun/Ambient.git ambient
```

You can also specify a specific version, like this;

```sh
cargo install --git https://github.com/AmbientRun/Ambient.git --tag v0.0.16 ambient
```

### Build dependencies: Linux/Ubuntu

For the above to work on linux, you also need to install the following build dependencies;

```sh
apt-get install -y build-essential cmake pkg-config libfontconfig1-dev clang libasound2-dev ninja-build
```

## Running on headless Linux/Ubuntu

To run on a headless linux machine, install the following dependencies;

```sh
add-apt-repository ppa:oibaf/graphics-drivers -y
apt-get update
apt install -y libxcb-xfixes0-dev mesa-vulkan-drivers
```

Ambient currently assumes that you have access to GPU drivers (but not necessarily a _GPU_ by itself) in headless mode. This requirement may be relaxed in future.
