use anyhow::Context;
use clap::{Args, ValueEnum};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum Target {
    /// Generates a wasm and js shim that uses `require` to import the `.wasm`
    Bundler,
    /// The shim won't import the `.wasm` itself, allowing for external fetching
    Standalone,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum Profile {
    Dev,
    Release,
    Profiling,
}

#[derive(Debug, Args, Clone)]
pub struct BuildOptions {
    #[arg(long, default_value = "dev")]
    pub(crate) profile: Profile,
    #[arg(long, default_value = "pkg")]
    pub pkg_name: String,
    #[arg(long, value_enum, default_value = "bundler")]
    target: Target,
}

impl BuildOptions {
    pub async fn check(&self) -> anyhow::Result<()> {
        let mut command = Command::new("cargo");

        command
            .args(["check", "-p", "ambient_web"])
            .current_dir("web")
            .kill_on_drop(true);

        match &self.profile {
            Profile::Dev => {
                // default
            }
            Profile::Release => {
                command.arg("--release");
            }
            Profile::Profiling => {
                command.args(["--profile=profiling"]);
            }
        };

        let res = command.spawn()?.wait().await?;

        if !res.success() {
            anyhow::bail!("checking package failed with status code: {res}");
        }

        Ok(())
    }

    pub async fn build(&self) -> anyhow::Result<PathBuf> {
        ensure_wasm_pack().await?;

        let mut command = Command::new("wasm-pack");

        command
            .args(["build", "client"])
            .current_dir("web")
            .kill_on_drop(true);

        match &self.profile {
            Profile::Dev => command.arg("--dev"),
            Profile::Release => command.arg("--release"),
            Profile::Profiling => command.arg("--profiling"),
        };

        match self.target {
            Target::Bundler => command.args(["--target", "bundler"]),
            Target::Standalone => command.args(["--target", "web", "--no-pack"]),
        };

        let mut output_path = ["web"]
            .iter()
            .collect::<PathBuf>()
            .canonicalize()
            .context("Produced build artifact does not exist")?;

        output_path.push(&self.pkg_name);

        command.arg("--out-dir").arg(output_path.clone());

        eprintln!("Building web client {command:?}");

        let res = command.spawn()?.wait().await?;

        if !res.success() {
            anyhow::bail!("Building package failed with status code: {res}");
        }

        assert!(output_path.exists());

        eprintln!("Built package: {:?}", output_path);

        Ok(output_path)
    }
}
#[cfg(not(target_os = "linux"))]
pub(crate) async fn install_wasm_pack() -> anyhow::Result<()> {
    eprintln!("Installing wasm-pack from source");
    let status = Command::new("cargo")
        .args(["install", "wasm-pack"])
        .kill_on_drop(true)
        .spawn()?
        .wait()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to install wasm-pack");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub(crate) async fn install_wasm_pack() -> anyhow::Result<()> {
    eprintln!("Installing wasm-pack");
    let mut curl = std::process::Command::new("curl")
        .args([
            "https://rustwasm.github.io/wasm-pack/installer/init.sh",
            "-sSf",
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn curl")?;

    let mut sh = std::process::Command::new("sh")
        .stdin(std::process::Stdio::from(curl.stdout.take().unwrap()))
        .spawn()
        .context("Failed to spawn sh")?;

    let sh = sh.wait()?;

    let curl = curl.wait()?;

    if !curl.success() {
        anyhow::bail!("Failed to fetch install script")
    }

    if !sh.success() {
        anyhow::bail!("Failed to run install script for wasm-pack")
    }

    Ok(())
}

pub async fn ensure_wasm_pack() -> anyhow::Result<()> {
    match which::which("wasm-pack") {
        Err(_) => {
            install_wasm_pack().await?;

            assert!(which::which("wasm-pack").is_ok(), "wasm-pack is in PATH");

            Ok(())
        }
        Ok(path) => {
            eprintln!("Found installation of wasm-pack at {path:?}");
            Ok(())
        }
    }
}
