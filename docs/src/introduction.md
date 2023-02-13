# Introduction

Kiwi Runtime provides a programming environment for building high performance games and 3d applications.

## Getting started

To create a project, first run;

```sh
kiwi new my-project
```

This will generate a new project for you. You can immediately run it:

```sh
cd my-project
kiwi run
```

From here on, you can open up the project in your favorite IDE and start editing the code.

The easiest way to get started is by looking at some of the [examples](https://github.com/KiwiOrg/Kiwi/tree/main/guest/rust/examples).

## API reference docs

To see the latest version of the API docs, run the following command in the `Kiwi` repository:

```sh
cd guest/rust
cargo doc -p kiwi_api --open --no-deps
```

## Setting up your IDE

We recommend using VSCode with the Rust Analyzer extension.

To work with the examples; open the `guest/rust` folder in VSCode to get code-completion etc.
