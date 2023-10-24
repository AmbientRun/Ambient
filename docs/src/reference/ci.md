# Setting up CI

## Github

Create a file called `.github/workflows/deploy.yaml` in your project, and with the following content:

```yaml
name: Deploy to Ambient
"on":
  push:
    branches:
      - main
jobs:
  build_and_deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          target: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - uses: actions-rs/install@v0.1
        with:
          crate: ambient
          version: latest
          use-tool-cache: true
      - run: ambient deploy --token ${{ secrets.AMBIENT_TOKEN }}
```

You will also need to set up the AMBIENT_TOKEN secret in your github project. You can get your token from your user project on http://ambient.run
