# Running examples

You can either run the examples from the latest released version of ambient,
or you can run the examples on main. _The important thing is that the
version of ambient matches what the examples were built for._

## Running examples from the latest release

1. Download the ambient executable from the [releases page](https://github.com/AmbientRun/Ambient/releases)
2. Download the `examples.zip` file from the same page
3. Extract both, and use that version of ambient to run the examples, for instance `./ambient run examples/basics/primitives`

## Running examples from main

1. Clone the github repository
2. Install ambient with `cargo install --path app ambient`
3. Then run the examples in the `guest/rust/example` directory, for instance: `ambient run guest/rust/examples/basics/primitives`
