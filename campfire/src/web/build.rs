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
    #[arg(long, value_enum, default_value = "bundler")]
    target: Target,
}

pub async fn run(opts: BuildOptions) -> anyhow::Result<()> {
    ensure_wasm_pack().await?;
    let output_path = run_cargo_build(&opts).await?;

    eprintln!("Built package: {:?}", output_path);

    Ok(())
}

pub async fn ensure_wasm_pack() -> anyhow::Result<()> {
    match which::which("wasm-pack") {
        Err(_) => {
            eprintln!("Installing wasm-pack");
            let status = Command::new("carg")
                .args(["install", "wasm-pack"])
                .spawn()?
                .wait()
                .await?;
            if !status.success() {
                anyhow::bail!("Failed to install wasm-pack");
            }

            Ok(())
        }
        Ok(path) => {
            eprintln!("Found installation of wasm pack at {path:?}");
            Ok(())
        }
    }
}

pub async fn run_cargo_build(opts: &BuildOptions) -> anyhow::Result<PathBuf> {
    let mut command = Command::new("wasm-pack");

    command.args(["build", "client"]).current_dir("web");

    match &opts.profile[..] {
        "dev" | "debug" => command.arg("--dev"),
        "release" => command.arg("--release"),
        v => anyhow::bail!("Unknown profile: {v:?}"),
    };

    match opts.target {
        Target::Bundler => command.args(["--target", "bundler"]),
        Target::Standalone => command.args(["--target", "web", "--no-pack"]),
    };

    // See: https://doc.rust-lang.org/cargo/guide/build-cache.html
    let output_path = ["web", "pkg"]
        .iter()
        .collect::<PathBuf>()
        .canonicalize()
        .context("Produced build artifact does not exist")?;

    command.arg("--out-dir").arg(output_path.clone());

    eprintln!("Building web client");

    let res = command.spawn()?.wait().await?;
    if !res.success() {
        anyhow::bail!("Building package failed with status code: {res}");
    }

    assert!(output_path.exists());

    Ok(output_path)
}
