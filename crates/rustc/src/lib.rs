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
    /// Gets the system installation of Rust, or downloads one if not available
    pub async fn install_or_get(rust_path: &Path) -> anyhow::Result<Self> {
        let installation = match Installation::System.get_installed_version() {
            Ok(version) if version >= MINIMUM_RUST_VERSION => {
                log::debug!("Using system rustc ({version})");
                Installation::System
            }
            _ if !rust_path.exists() => {
                log::debug!("No rustc detected, downloading and installing");
                Installation::download_and_install(rust_path).await?
            }
            _ => {
                let installation = Installation::from_existing_installation(rust_path)?;
                let mut version = installation.get_installed_version()?;

                if version < MINIMUM_RUST_VERSION {
                    log::debug!("Downloaded rustc is out of date ({version}), updating");
                    installation.update()?;
                    version = installation.get_installed_version()?;
                }

                log::debug!("Using downloaded rustc ({version})");
                installation
            }
        };
        installation.install_wasm32_wasi()?;

        Ok(Self(installation))
    }

    pub fn build(&self, working_directory: &Path, package_name: &str) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(
            parse_command_result_for_filenames(self.0.run(
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
                Some(working_directory),
            ))?
            .into_iter()
            .find(|p| p.extension().unwrap_or_default() == "wasm")
            .context("no wasm artifact")?,
        )?)
    }

    pub fn document(&self, module_path: &Path) -> anyhow::Result<PathBuf> {
        Ok(parse_command_result_for_filenames(self.0.run(
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
            Some(module_path),
        ))?
        .into_iter()
        .find(|p| p.extension().unwrap_or_default() == "html")
        .context("no html artifact")?
        .parent()
        .and_then(Path::parent)
        .context("no parent")?
        .to_owned())
    }
}

#[derive(Clone)]
enum Installation {
    Downloaded {
        rustup_path: PathBuf,
        cargo_path: PathBuf,
    },
    System,
}
impl Installation {
    async fn download_and_install(rust_path: &Path) -> anyhow::Result<Self> {
        log::info!("Downloading rustup");

        std::fs::create_dir_all(rust_path).context("failed to create rust_path")?;
        let (rustup_path, cargo_path) = Self::paths(rust_path);
        std::fs::create_dir_all(&rustup_path).context("failed to create rustup_path")?;
        std::fs::create_dir_all(&cargo_path).context("failed to create cargo_path")?;

        let target = env!("TARGET").to_string();

        // HACK(philpax): the -msvc toolchain requires msvc build tools, which rustup is not guaranteed
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

        let rustup_init_path = rust_path.join("rustup-init");
        std::fs::write(&rustup_init_path, contents)?;
        let _rustup_init_deleter = DeleteOnExit::new(rustup_init_path.clone());

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&rustup_init_path, std::fs::Permissions::from_mode(0o777))
                .context("failed to set rustup-init permissions")?;
        }

        log::info!("Executing rustup");
        let mut rust_deleter = DeleteOnExit::new(rust_path.to_owned());
        let mut command = Command::new(&rustup_init_path);
        silence_output_window(&mut command);
        let result = command
            .envs([
                ("RUSTUP_HOME", rustup_path.to_string_lossy().as_ref()),
                ("CARGO_HOME", cargo_path.to_string_lossy().as_ref()),
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

        let installation = Installation::Downloaded {
            rustup_path,
            cargo_path,
        };

        log::info!("Installing wasm32-wasi");
        installation.install_wasm32_wasi()?;

        log::info!("Setting rustup default");
        handle_command_failure(
            "set rustup default",
            installation.run("rustup", ["default", "stable"], None),
        )?;

        installation.update().context("failed to update rust")?;
        rust_deleter.disable_deletion();

        Ok(installation)
    }

    fn from_existing_installation(rust_path: &Path) -> anyhow::Result<Self> {
        let (rustup_path, cargo_path) = Self::paths(rust_path);

        if !rustup_path.is_dir() {
            anyhow::bail!("rustup path {rustup_path:?} is not a directory or does not exist");
        }

        if !cargo_path.is_dir() {
            anyhow::bail!("cargo path {cargo_path:?} is not a directory or does not exist");
        }

        Ok(Self::Downloaded {
            rustup_path,
            cargo_path,
        })
    }

    fn paths(rust_path: &Path) -> (PathBuf, PathBuf) {
        let rustup_path = rust_path.join("rustup");
        let cargo_path = rust_path.join("cargo");

        (rustup_path, cargo_path)
    }

    /// Should only be called on downloaded installations. Will assert if called on a system
    /// installation.
    fn update(&self) -> anyhow::Result<()> {
        if !matches!(&self, Installation::Downloaded { .. }) {
            anyhow::bail!("rustup update should only be called on downloaded installations");
        }

        log::info!("Running rustup update");
        handle_command_failure("run rustup update", self.run("rustup", ["update"], None))?;

        Ok(())
    }

    fn install_wasm32_wasi(&self) -> anyhow::Result<()> {
        handle_command_failure(
            "add rustup target wasm32-wasi",
            self.run("rustup", ["target", "add", "wasm32-wasi"], None),
        )?;

        Ok(())
    }

    fn get_installed_version(&self) -> anyhow::Result<Version> {
        Ok(Version(
            handle_command_failure("get version", self.run("rustc", ["--version"], None))?
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
        let exe_path = match self {
            Installation::Downloaded { cargo_path, .. } => cargo_path.join("bin").join(exe(cmd)),
            Installation::System => PathBuf::from(exe(cmd)),
        };

        let mut command = Command::new(exe_path);
        silence_output_window(&mut command);

        let mut envs = vec![
            ("RUSTUP_TOOLCHAIN", "stable".to_string()),
            ("CARGO_INCREMENTAL", "1".to_string()),
        ];
        if let Installation::Downloaded {
            rustup_path,
            cargo_path,
        } = self
        {
            envs.extend_from_slice(&[
                ("RUSTUP_HOME", rustup_path.to_string_lossy().to_string()),
                ("CARGO_HOME", cargo_path.to_string_lossy().to_string()),
            ]);
        }

        command.envs(envs).args(args);
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

struct DeleteOnExit {
    path: PathBuf,
    should_delete: bool,
}
impl DeleteOnExit {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            should_delete: true,
        }
    }

    fn disable_deletion(&mut self) {
        self.should_delete = false;
    }
}
impl Drop for DeleteOnExit {
    fn drop(&mut self) {
        if !self.should_delete {
            return;
        }

        if !self.path.exists() {
            return;
        }

        if self.path.is_file() {
            std::fs::remove_file(&self.path).ok();
        } else if self.path.is_dir() {
            std::fs::remove_dir_all(&self.path).ok();
        }
    }
}
