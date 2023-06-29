# Setting up your IDE

Our recommended IDE is Visual Studio Code (VSCode).

## Visual Studio Code (VSCode)

Install [Visual Studio Code](https://code.visualstudio.com/), then install the following plugins:

- [rust-analyzer](https://rust-analyzer.github.io/), as described [here](https://code.visualstudio.com/docs/languages/rust).
- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb). This one is optional, but with it you can launch your ember from with VSCode by pressing F5.

`ambient new` will set up your ember for VSCode by default, by creating a `.vscode/settings.json` for you.

## Emacs

There are multiple ways to configure Emacs as a Rust IDE. The following assumes you are using [rustic](https://github.com/brotzeit/rustic),
[lsp-mode](https://github.com/emacs-lsp/lsp-mode) and [rust-analyzer](https://rust-analyzer.github.io/) libraries. [Robert Krahn provides a comprehensive guide to configuring Emacs for Rust development](https://robert.kra.hn/posts/rust-emacs-setup/#prerequisites)

Once you have Emacs configured for general Rust development, you need to set some explicit values for Ambient embers. Ambient uses some custom `cargo` configuration values that Emacs and rust-analyzer need to know about. You can manually set these variables with the following `elisp`:

```elisp
  (setq lsp-rust-analyzer-cargo-target "wasm32-wasi"
        lsp-rust-analyzer-cargo-watch-args ["--features" "client server"]
        lsp-rust-features ["client" "server"])
```

Furthermore, you can add a `.dir-locals.el` file to your Ambient ember directory that Emacs will pick up and load settings for. This is similar to the `.vscode/settings.json` that is created by default. This is an example `.dir-locals.el` file:

```elisp
((rustic-mode . ((eval . (setq-local lsp-rust-analyzer-cargo-target "wasm32-wasi"))
                 (eval . (setq-local lsp-rust-analyzer-cargo-watch-args ["--features" "client server"]))
                 (eval . (setq-local lsp-rust-features ["client" "server"])))))
```

## Other IDEs

To get rust-analyzer to work, you need to make sure it's building with the `server` and `client` feature flags enabled. See [.vscode/settings.json](https://github.com/AmbientRun/Ambient/blob/main/app/src/cli/new_project_template/.vscode/settings.json) for an example.
