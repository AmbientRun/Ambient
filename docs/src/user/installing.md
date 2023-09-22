# Installing

Native use of Ambient, for both developing and playing games, is easy. We have a version manager that
will retrieve a pre-built version of Ambient for your platform. This is the recommended way to use Ambient.

The steps are as follows, where the commands are for your terminal of choice:

1.  [Install Rust](https://www.rust-lang.org/). Note that the minimum supported version is 1.71.0, and you may need to update.
2.  Add the `wasm32-wasi` toolchain. This lets you compile Rust code for Ambient.

        rustup target add --toolchain stable wasm32-wasi

3.  Install the Ambient version manager:

        cargo install ambient

The native client of Ambient currently runs on Windows, Linux and macOS.

Next, try the [tutorial](../tutorials/game/0_intro.md) to create your first Ambient game!
