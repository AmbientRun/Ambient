# Installing

You need [Rust](https://www.rust-lang.org/) to install the Tilt runtime. Then run:

```sh
cargo install tilt
```

## Dependencies: Linux/Ubuntu

```sh
apt-get install -y build-essential cmake pkg-config \
  libfontconfig1-dev clang libasound2-dev ninja-build
```

## Running on headless Linux/Ubunutu

```sh
add-apt-repository ppa:oibaf/graphics-drivers -y
apt-get update
apt install -y libxcb-xfixes0-dev mesa-vulkan-drivers
```
