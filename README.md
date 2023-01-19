# Elements

## Installing

```sh
cargo install tilt
```

### Dependencies: Linux/Ubuntu

```sh
apt-get install -y build-essential cmake pkg-config libfontconfig1-dev clang libasound2-dev ninja-build
```

## Running on headless Linux/Ubunutu

```sh
add-apt-repository ppa:oibaf/graphics-drivers -y
apt-get update
apt install -y libxcb-xfixes0-dev mesa-vulkan-drivers
```
