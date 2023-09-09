# Installing

The easiest way to install Ambient is to download the latest binary release from the [GitHub releases](https://github.com/AmbientRun/Ambient/releases).
Currently, there are builds for Windows x64, Linux x64, and Mac ARM64. For other platforms, use the [installing from Git](#installing-from-git) method.

You will also need [Rust](https://www.rust-lang.org/) installed with the `wasm32-wasi` toolchain so that you can compile Ambient modules.
Note that Ambient compiles Rust code with stable Rust, so you must make sure that `wasm32-wasi` is installed for `stable`:

```sh
rustup target add --toolchain stable wasm32-wasi
```

## Installing from Git

Ambient can be installed through `cargo install`.

This will automatically download the source, compile and install Ambient from your system. Our minimum supported Rust version is <!-- rust-version-begin -->1.70.0<!-- rust-version-end -->.

### Installing the latest published release

This is the recommended method of installing Ambient from source if the downloadable binaries are insufficient. The latest published release should be used unless you have a specific reason to use the development version.

```sh
cargo install --git https://github.com/AmbientRun/Ambient.git --tag v0.3.0-dev ambient
```

### Installing the latest development version

Ambient is actively developed on the `main` branch of the repository. This branch contains in-development changes, including new features, bug fixes and breaking changes. This method can be used if you would like to try out these changes.

**Note**: The `main` branch is subject to frequent breaking changes, including potential new bugs and decreased stability, and is not a stable development target for packages. Using the `main` branch is not recommended if you are unable to actively update your package to accommodate breaking changes.

```sh
cargo install --git https://github.com/AmbientRun/Ambient.git --locked --force ambient
```

**Note**: If you are running a package outside of the `guest/rust` workspace, it is likely that the published version of the API will be incompatible with `main`, and you will need to specify the dependency manually.

Additionally, the `--locked` flag is recommended to ensure that the correct packages are installed and that the build is reproducible between machines.

### Optional features

You can supply these feature flags to get optional features that are disabled by default:

```sh
cargo install --git https://github.com/AmbientRun/Ambient.git ambient --features assimp --locked --force
```

- `assimp`: This adds support for [assimp](https://github.com/assimp/assimp), which loads ~40 additional model file formats, such as `obj`, text-based `fbx` and much more

### Build dependencies: Linux/Ubuntu

For the above to work on Linux, you also need to install the following build dependencies:

```sh
apt-get install -y \
    build-essential cmake pkg-config \
    libfontconfig1-dev clang libasound2-dev ninja-build
```

## Installing via asdf (Linux, Macos)

Thanks to [@jtakakura](https://github.com/jtakakura), Ambient can also be installed using [asdf](https://asdf-vm.com/) by running `asdf plugin add ambient`. For more details, visit <https://github.com/jtakakura/asdf-ambient>.

## Running on headless Linux/Ubuntu

To run on a headless Linux machine, install the following dependencies in addition to the dependencies specified above:

```sh
add-apt-repository ppa:oibaf/graphics-drivers -y
apt-get update
apt install -y libxcb-xfixes0-dev mesa-vulkan-drivers
```

Ambient currently assumes that you have access to GPU drivers (but not necessarily a GPU) in headless mode. This requirement may be relaxed in future.

## Dockerfile

A `Dockerfile` is also provided that provides a headless Debian environment with all of the dependencies required to run Ambient as a server. This Dockerfile is intended for development, not production, so it has more dependencies than are strictly required to run Ambient.

To build the Dockerfile:

```sh
docker build -t ambient .
```

To run the Dockerfile with `bash` in the current directory:

```sh
docker run --rm -it -e bash -v "$(pwd)":/app ambient
```
