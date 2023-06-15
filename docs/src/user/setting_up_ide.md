
# Setting up your IDE

Our recommended setup is using Visual studio code (VSCode).

## Visual Studio Code (VSCode)

First, install [Visual Studio Code](https://code.visualstudio.com/). Then install the following plugins:
- [rust-analyzer](https://rust-analyzer.github.io/), as described [here](https://code.visualstudio.com/docs/languages/rust).
- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb). This one is optional, but with it you can launch your project from with VSCode by pressing F5.

`ambient new` will by default set up your project for VSCode, by creating this [.vscode/settings.json](https://github.com/AmbientRun/Ambient/blob/main/app/src/cli/new_project_template/.vscode/settings.json) file for you.

## Other IDEs

To get rust-analyzer to work, you need to make sure it's building with the `server` and `client` feature flags enabled. See [.vscode/settings.json](https://github.com/AmbientRun/Ambient/blob/main/app/src/cli/new_project_template/.vscode/settings.json) for an example.
