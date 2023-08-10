# Running examples

You can either run the examples from the latest released version of Ambient, or with the development `main` branch.

However, **the version of Ambient must match the version that the examples were built for**. For instance, if you are running the `main` branch of Ambient, you must also run the `main` branch of the examples.

## Running examples from the latest release

1. Download the Ambient executable from the [releases page](https://github.com/AmbientRun/Ambient/releases).
2. Download the `examples.zip` file from the same page.
3. Extract both, and use the extracted Ambient to run the examples: `./ambient run examples/basics/primitives`

## Running examples from main

1. Clone the GitHub repository.
2. Install Ambient with `cargo install --path app ambient`.
3. Run the examples in the `guest/rust/example` directory: `ambient run guest/rust/examples/basics/primitives`
