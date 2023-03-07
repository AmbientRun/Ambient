use std::{
    ffi::OsStr,
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::Context;
use itertools::Itertools;

const MINIMUM_RUST_VERSION: Version = Version((1, 65, 0));

#[derive(Clone)]
pub struct Rust(Installation);
impl Rust {
    pub async fn get_system_installation() -> anyhow::Result<Self> {
        let installation = Installation;
        if installation.get_installed_rustup_version().is_err() {
            anyhow::bail!("`rustup` is not installed. Please install it with https://rustup.rs/ for the best experience.");
        }

        if !installation
            .get_installed_rustc_version()
            .map(|v| v >= MINIMUM_RUST_VERSION)
            .unwrap_or(false)
        {
            anyhow::bail!("`rustc` is not installed. Please install it with `rustup` for the best experience.");
        }

        if !installation
            .get_installed_targets()?
            .iter()
            .any(|s| s == "wasm32-wasi")
        {
            anyhow::bail!("Your `rustup` installation does not have `wasm32-wasi` installed for the stable toolchain. Please install it with `rustup target add --toolchain stable wasm32-wasi`.")
        }

        Ok(Self(installation))
    }

    pub fn build(
        &self,
        working_directory: &Path,
        package_name: &str,
        optimize: bool,
    ) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(
            parse_command_result_for_filenames(
                self.0.run(
                    "cargo",
                    [
                        "build",
                        if optimize { "--release" } else { "" },
                        "--message-format",
                        "json",
                        "--target",
                        "wasm32-wasi",
                        "--package",
                        package_name,
                    ]
                    .into_iter()
                    .filter(|a| !a.is_empty()),
                    Some(working_directory),
                ),
            )?
            .into_iter()
            .find(|p| p.extension().unwrap_or_default() == "wasm")
            .context("no wasm artifact")?,
        )?)
    }
}

#[derive(Clone)]
struct Installation;
impl Installation {
    fn get_installed_rustup_version(&self) -> anyhow::Result<Version> {
        self.get_version_for("get rustup version", "rustup")
    }

    fn get_installed_rustc_version(&self) -> anyhow::Result<Version> {
        self.get_version_for("get rustc version", "rustc")
    }

    fn get_installed_targets(&self) -> anyhow::Result<Vec<String>> {
        Ok(
            handle_command_failure("get targets", self.run("rustup", ["target", "list"], None))?
                .lines()
                .filter_map(|l| Some(l.strip_suffix("(installed)")?.trim().to_string()))
                .collect(),
        )
    }

    fn get_version_for(&self, task: &str, cmd: &str) -> Result<Version, anyhow::Error> {
        Ok(Version(
            handle_command_failure(task, self.run(cmd, ["--version"], None))?
                .split_whitespace()
                .nth(1)
                .context("failed to extract version component (1)")?
                .split('-')
                .next()
                .context("failed to extract version component (2)")?
                .split('.')
                .map(|i| i.parse().unwrap_or_default())
                .collect_tuple()
                .context("failed to collect version into tuple")?,
        ))
    }

    fn run(
        &self,
        cmd: &str,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
        working_directory: Option<&Path>,
    ) -> anyhow::Result<(bool, String, String)> {
        let exe_path = PathBuf::from(exe(cmd));

        let mut command = Command::new(exe_path);
        silence_output_window(&mut command);

        command
            .envs([
                ("RUSTUP_TOOLCHAIN", "stable".to_string()),
                ("CARGO_INCREMENTAL", "1".to_string()),
            ])
            .args(args);
        if let Some(wd) = working_directory {
            command.current_dir(wd);
        }
        let result = command.output()?;

        Ok((
            result.status.success(),
            std::str::from_utf8(&result.stdout)?.to_owned(),
            std::str::from_utf8(&result.stderr)?.to_owned(),
        ))
    }
}

fn parse_command_result_for_filenames(
    result: anyhow::Result<(bool, String, String)>,
) -> anyhow::Result<Vec<PathBuf>> {
    let (success, stdout, stderr) = result?;

    let messages: Vec<_> = stdout
        .lines()
        .filter_map(|l| Some(serde_json::Value::from_str(l).ok()?.as_object()?.to_owned()))
        .filter(|v| {
            let reason = v.get("reason").and_then(|v| v.as_str()).unwrap_or_default();
            reason == "compiler-artifact" || reason == "compiler-message"
        })
        .collect();

    if success {
        let last_compiler_artifact = messages
            .iter()
            .rfind(|v| v.get("reason").and_then(|v| v.as_str()) == Some("compiler-artifact"))
            .context("no compiler-artifact")?;
        let filenames = last_compiler_artifact
            .get("filenames")
            .and_then(|f| f.as_array())
            .context("no filenames")?;
        Ok(filenames
            .iter()
            .filter_map(|s| s.as_str())
            .map(|p| p.into())
            .collect())
    } else {
        let stdout_errors = messages
            .iter()
            .filter(|v| v.get("reason").and_then(|v| v.as_str()) == Some("compiler-message"))
            .map(|v| {
                v.get("message")
                    .and_then(|m| m.as_object())
                    .and_then(|m| m.get("rendered"))
                    .and_then(|r| r.as_str())
                    .unwrap_or_default()
            })
            .join("");

        anyhow::bail!(
            "failed to compile, {}",
            generate_error_report(stdout_errors, stderr)
        );
    }
}

fn handle_command_failure(
    task: &str,
    result: anyhow::Result<(bool, String, String)>,
) -> anyhow::Result<String> {
    let (success, stdout, stderr) = result?;
    if !success {
        anyhow::bail!(
            "failed to {task}: {}",
            generate_error_report(stdout, stderr)
        )
    }
    Ok(stdout)
}

fn generate_error_report(stdout: String, stderr: String) -> String {
    [("stdout", stdout), ("stderr", stderr)]
        .into_iter()
        .filter(|(_, errors)| !errors.is_empty())
        .map(|(name, errors)| format!("{name}: {errors}"))
        .join(", ")
}

fn exe(app: &str) -> String {
    format!(
        "{app}{}",
        env!("TARGET")
            .contains("windows")
            .then_some(".exe")
            .unwrap_or_default()
    )
}

#[cfg(target_os = "windows")]
fn silence_output_window(command: &mut Command) {
    // https://stackoverflow.com/a/60764548
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x08000000);
}

#[cfg(not(target_os = "windows"))]
fn silence_output_window(_: &mut Command) {}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Version((u32, u32, u32));
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (major, minor, patch) = self.0;
        write!(f, "{major}.{minor}.{patch}")
    }
}
