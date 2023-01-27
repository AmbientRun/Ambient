use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::Context;
use itertools::Itertools;
use log::info;

#[derive(Clone)]
pub struct InstallDirs {
    pub rustup_path: PathBuf,
    pub cargo_path: PathBuf,
}

pub async fn download_and_install(
    dirs: &InstallDirs,
    rustup_init_path: &Path,
) -> anyhow::Result<()> {
    info!("downloading rustup");
    std::fs::create_dir_all(&dirs.rustup_path).context("failed to create rustup_path")?;
    std::fs::create_dir_all(&dirs.cargo_path).context("failed to create cargo_path")?;

    let target = env!("TARGET").to_string();

    // HACK(mithun): the -msvc toolchain requires msvc build tools, which rustup is not guaranteed
    // to install. instead, we force the -gnu toolchain, which shouldn't require those tools:
    // x86_64-pc-windows-msvc -> x86_64-pc-windows-gnu
    #[cfg(target_os = "windows")]
    let target = target.replace("-msvc", "-gnu");

    let url = format!(
        "https://static.rust-lang.org/rustup/dist/{}/{}",
        target,
        exe("rustup-init")
    );

    let contents = reqwest::get(url)
        .await?
        .bytes()
        .await
        .context("failed to download rustup-init")?;
    std::fs::write(rustup_init_path, contents)?;

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(rustup_init_path, std::fs::Permissions::from_mode(0o777))
            .context("failed to set rustup-init permissions")?;
    }

    info!("executing rustup");
    let mut command = Command::new(rustup_init_path);
    silence_output_window(&mut command);
    let result = command
        .envs([
            ("RUSTUP_HOME", dirs.rustup_path.to_string_lossy().as_ref()),
            ("CARGO_HOME", dirs.cargo_path.to_string_lossy().as_ref()),
        ])
        .args([
            "-y",
            "--no-update-default-toolchain",
            "--no-modify-path",
            "-t",
            "wasm32-wasi",
        ])
        .output()
        .context("failed to run rustup-init")?;
    if !result.status.success() {
        anyhow::bail!(
            "failed to execute rustup-init: {}",
            std::str::from_utf8(&result.stderr)?
        );
    }
    std::fs::remove_file(rustup_init_path).context("failed to remove rustup-init")?;

    info!("installing wasm32-wasi");
    handle_command_failure(
        "add rustup target wasm32-wasi",
        run_command(dirs, "rustup", ["target", "add", "wasm32-wasi"], None),
    )?;

    info!("setting rustup default");
    handle_command_failure(
        "set rustup default",
        run_command(dirs, "rustup", ["default", "stable"], None),
    )?;

    update_rust(dirs).context("failed to update rust")?;

    Ok(())
}

pub fn update_rust(dirs: &InstallDirs) -> anyhow::Result<()> {
    info!("running rustup update");
    handle_command_failure(
        "run rustup update",
        run_command(dirs, "rustup", ["update"], None),
    )?;

    Ok(())
}

pub fn build_module_in_workspace(
    dirs: &InstallDirs,
    workspace_path: &Path,
    package_name: &str,
) -> anyhow::Result<Vec<u8>> {
    Ok(std::fs::read(
        parse_command_result_for_filenames(run_command(
            dirs,
            "cargo",
            [
                "build",
                "--release",
                "--message-format",
                "json",
                "--target",
                "wasm32-wasi",
                "--package",
                package_name,
            ],
            Some(workspace_path),
        ))?
        .into_iter()
        .find(|p| p.extension().unwrap_or_default() == "wasm")
        .context("no wasm artifact")?,
    )?)
}

pub fn document_module(dirs: &InstallDirs, script_path: &Path) -> anyhow::Result<PathBuf> {
    Ok(parse_command_result_for_filenames(run_command(
        dirs,
        "cargo",
        [
            "doc",
            "--release",
            "--message-format",
            "json",
            "--no-deps",
            "--target",
            "wasm32-wasi",
        ],
        Some(script_path),
    ))?
    .into_iter()
    .find(|p| p.extension().unwrap_or_default() == "html")
    .context("no html artifact")?
    .parent()
    .and_then(Path::parent)
    .context("no parent")?
    .to_owned())
}

pub fn get_installed_version(dirs: &InstallDirs) -> anyhow::Result<(u32, u32, u32)> {
    let version = handle_command_failure(
        "get version",
        run_command(dirs, "rustc", ["--version"], None),
    )?;
    version
        .split_whitespace()
        .nth(1)
        .context("failed to extract version component (1)")?
        .split('-')
        .next()
        .context("failed to extract version component (2)")?
        .split('.')
        .map(|i| i.parse().unwrap_or_default())
        .collect_tuple::<(u32, u32, u32)>()
        .context("failed to collect version into tuple")
}

fn run_command(
    dirs: &InstallDirs,
    cmd: &str,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    working_directory: Option<&Path>,
) -> anyhow::Result<(bool, String, String)> {
    let exe_path = dirs.cargo_path.join("bin").join(exe(cmd));

    let mut command = Command::new(exe_path);
    silence_output_window(&mut command);
    command
        .envs([
            ("RUSTUP_HOME", dirs.rustup_path.to_string_lossy().as_ref()),
            ("CARGO_HOME", dirs.cargo_path.to_string_lossy().as_ref()),
            ("RUSTUP_TOOLCHAIN", "stable"),
            ("CARGO_INCREMENTAL", "1"),
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
