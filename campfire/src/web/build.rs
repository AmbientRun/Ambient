use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use tokio::process::Command;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum Target {
    /// Generates a wasm and js shim that uses `require` to import the `.wasm`
    Bundler,
    /// The shim won't import the `.wasm` itself, allowing for external fetching
    Standalone,
}

#[derive(Debug, Args, Clone)]
pub struct BuildOptions {
    #[arg(long, default_value = "dev")]
    pub profile: String,
    #[arg(long, value_enum)]
    target: Target,
}

pub async fn run(opts: BuildOptions) -> anyhow::Result<()> {
    let output_path = run_cargo_build(&opts).await?;

    run_wasm_bindgen(&output_path, &opts).await?;

    Ok(())
}

pub async fn run_cargo_build(opts: &BuildOptions) -> anyhow::Result<PathBuf> {
    let mut command = Command::new("cargo");

    command
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--profile",
            &opts.profile,
        ])
        .current_dir("web");

    eprintln!("Building web client");
    let res = command.spawn()?.wait().await?;
    if !res.success() {
        anyhow::bail!("Building package failed with status code: {res}");
    }

    // See: https://doc.rust-lang.org/cargo/guide/build-cache.html
    let output_path = [
        "web",
        "target",
        "wasm32-unknown-unknown",
        match &opts.profile[..] {
            "dev" => "debug",
            v => v,
        },
        "ambient_web.wasm",
    ]
    .iter()
    .collect::<PathBuf>()
    .canonicalize()
    .context("Produced build artifact does not exist")?;

    assert!(output_path.exists());

    eprintln!("Built package: {:?}", output_path);

    Ok(output_path)
}

pub async fn run_wasm_bindgen(
    path: impl AsRef<Path>,
    opts: &BuildOptions,
) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();

    eprintln!("Generating wasm bindings");

    let mut command = Command::new("wasm-bindgen");

    let output_path = ["web", "pkg"].iter().collect::<PathBuf>();

    command.args(["--out-dir"]).arg(&output_path).arg(path);

    match opts.target {
        Target::Bundler => command.args(["--target", "bundler"]),
        Target::Standalone => command.args(["--target", "no-modules"]),
    };

    let res = command.spawn()?.wait().await?;
    if !res.success() {
        anyhow::bail!("Generating wasm bindings for package failed with status code: {res}");
    }

    let output_path = output_path.canonicalize()?;

    eprintln!("Generated wasm package at: {:?}", output_path);
    Ok(output_path)
}
