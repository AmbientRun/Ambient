
# Setting up your IDE

Our recommended setup is using Visual studio code (VSCode).

## Visual Studio Code (VSCode)

First, install [Visual Studio Code](https://code.visualstudio.com/). Then install the following plugins:
- [rust-analyzer](https://rust-analyzer.github.io/), as described [here](https://code.visualstudio.com/docs/languages/rust).
- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb). This one is optional, but with it you can launch your project from with VSCode by pressing F5.

`ambient new` will by default set up your project for VSCode, by creating this [.vscode/settings.json](https://github.com/AmbientRun/Ambient/blob/main/app/src/cli/new_project_template/.vscode/settings.json) file for you.

## Emacs

There are multiple ways to configure Emacs as a Rust IDE. The following assumes you are using [rustic](https://github.com/brotzeit/rustic),
[lsp-mode](https://github.com/emacs-lsp/lsp-mode) and [rust-analyzer](https://rust-analyzer.github.io/) libraries. [Robert Krahn provides a comprehensive guide to configuring Emacs for Rust development](https://robert.kra.hn/posts/rust-emacs-setup/#prerequisites)

Once you have Emacs configured for general Rust development. You need to set some explicit values for Ambient projects. Ambient uses some custom `cargo` configuration values that Emacs and rust-analyzer need to know about you can manually set these variables with the following elisp:


``` elisp
  (setq lsp-rust-analyzer-cargo-target "wasm32-wasi"
        lsp-rust-analyzer-cargo-watch-args ["--features" "client server"]
        lsp-rust-features ["client" "server"])
```


Furthermore you can add a `.dir-locals.el` file to your Ambient project directory that Emacs will pick up and load settings for. Similiar to the `.vscode/settings.json` an example of that file for an Ambient project:

``` elisp
((rustic-mode . ((eval . (setq-local lsp-rust-analyzer-cargo-target "wasm32-wasi"))
                 (eval . (setq-local lsp-rust-analyzer-cargo-watch-args ["--features" "client server"]))
                 (eval . (setq-local lsp-rust-features ["client" "server"])))))
```

## Other IDEs

To get rust-analyzer to work, you need to make sure it's building with the `server` and `client` feature flags enabled. See [.vscode/settings.json](https://github.com/AmbientRun/Ambient/blob/main/app/src/cli/new_project_template/.vscode/settings.json) for an example.
