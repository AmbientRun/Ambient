# Installing

The easiest way to install Ambient is to download the latest binary release from the [GitHub releases](https://github.com/AmbientRun/Ambient/releases).
Currently, there are builds for Windows x64, Linux x64, and Mac ARM64. For other platforms, use the [installing from Git](#installing-from-git) method.

You will also need [Rust](https://www.rust-lang.org/) installed with the `wasm32-wasi` toolchain so that you can compile Ambient modules.
Note that Ambient compiles Rust code with stable Rust, so you must make sure that `wasm32-wasi` is installed for `stable`:

```sh
rustup target add --toolchain stable wasm32-wasi
```

## Installing from Git

Ambient can be installed through `cargo install`, which will download and build the repository. Our current minimum supported Rust version is 1.67.0, as we use recently-stabilised standard library features.

To install the latest released version from Git, run the following:

```sh
cargo install --git https://github.com/AmbientRun/Ambient.git --tag v0.1.1
```

To install the latest version on the `main` branch, run the following:

```sh
cargo install --git https://github.com/AmbientRun/Ambient.git
```

Note that if you are running a project outside of the `guest/rust` workspace, it is likely that the published version of the API will be incompatible with `main`, and you will need to specify the dependency manually.

### Building problems and solutions

- If you run into troubles building russimp/assimp, you can turn it off by installing with `--no-default-features`. See [this issue](https://github.com/AmbientRun/Ambient/issues/173) for more details.

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
