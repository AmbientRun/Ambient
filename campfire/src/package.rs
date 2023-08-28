use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Args, Parser};
use itertools::Itertools;

use crate::util::{all_directories_in, run_ambient};

#[derive(Parser, Clone)]
#[clap(trailing_var_arg = true)]
pub enum Package {
    /// Clean all package build artifacts
    Clean,
    /// Run an package
    Run(Run),
    /// Run all the packages in order
    RunAll(RunParams),
    /// Check all the packages
    CheckAll,
    /// Build all standard packages
    BuildAll,
    /// Publish all standard packages
    DeployAll {
        #[arg(long)]
        token: String,
        #[arg(long)]
        include_examples: bool,
    },
}

#[derive(Parser, Clone)]
#[clap(trailing_var_arg = true)]
/// Run a package
pub struct Run {
    /// The name of the package to run
    pub package: String,
    #[command(flatten)]
    pub params: RunParams,
}

#[derive(Args, Clone, Debug)]
#[clap(trailing_var_arg = true)]
/// Run a package
pub struct RunParams {
    /// Whether or not to run Ambient in release mode
    #[arg(short, long, default_value_t = false)]
    pub release: bool,
    /// The args to pass through to `ambient`
    pub args: Vec<String>,
}

pub fn main(args: &Package) -> anyhow::Result<()> {
    match args {
        Package::Clean => clean(),
        Package::Run(args) => run(args),
        Package::RunAll(params) => run_all(params),
        Package::CheckAll => check_all(),
        Package::BuildAll => build_all(),
        Package::DeployAll {
            token,
            include_examples,
        } => deploy_all(token, *include_examples),
    }
}

pub fn build_all() -> anyhow::Result<()> {
    let package_paths = get_all_packages(true, true)?;

    for path in &package_paths {
        run_ambient(&["build", &path.to_string_lossy(), "--clean-build"], true)?;
    }

    Ok(())
}

pub fn deploy_all(token: &str, include_examples: bool) -> anyhow::Result<()> {
    let paths = get_all_packages(include_examples, false)?;
    let paths = paths.iter().map(|p| p.to_string_lossy()).collect_vec();

    let mut args = vec!["deploy"];
    for (idx, path) in paths.iter().enumerate() {
        if idx != 0 {
            args.push("--extra-packages");
        }
        args.push(&path);
    }
    args.push("--token");
    args.push(token);
    args.push("--clean-build");

    run_ambient(&args, true)
}

pub fn clean() -> anyhow::Result<()> {
    log::info!("Cleaning examples...");
    for path in get_all_packages(true, true)? {
        let build_path = path.join("build");
        if !build_path.exists() {
            continue;
        }

        std::fs::remove_dir_all(&build_path)?;
        log::info!("Removed build directory for {}.", path.display());
    }
    log::info!("Done cleaning examples.");
    Ok(())
}

pub fn run(args: &Run) -> anyhow::Result<()> {
    let Run { package, params } = args;

    let path = get_all_packages(true, true)?
        .into_iter()
        .find(|p| p.ends_with(package))
        .ok_or_else(|| anyhow::anyhow!("no example found with name {}", package))?;

    log::info!("Running example {} (params: {params:?})...", path.display());
    run_package(&path, params)
}

fn run_all(params: &RunParams) -> anyhow::Result<()> {
    for path in get_all_packages(true, true)? {
        log::info!("Running example {} (params: {params:?})...", path.display());
        run_package(&path, params)?;
    }

    Ok(())
}

fn check_all() -> anyhow::Result<()> {
    // Rust
    {
        let root_path = Path::new("guest/rust");
        log::info!("Checking Rust guest code...");

        for features in ["", "client", "server", "client,server"] {
            log::info!("Checking Rust guest code with features `{}`...", features);

            let mut command = std::process::Command::new("cargo");
            command.current_dir(root_path);
            command.args(["clippy"]);
            command.env("RUSTFLAGS", "-Dwarnings");

            if !features.is_empty() {
                command.args(["--features", features]);
            }

            if !command.spawn()?.wait()?.success() {
                anyhow::bail!(
                    "Failed to check Rust guest code with features {}.",
                    features
                );
            }
        }

        log::info!("Checked Rust guest code.");
    }

    Ok(())
}

fn run_package(path: &Path, params: &RunParams) -> anyhow::Result<()> {
    let mut args = vec!["run"];
    let path = path.to_string_lossy();
    args.push(&path);
    if !params.args.is_empty() {
        args.extend(params.args.iter().map(|s| s.as_str()));
    }
    run_ambient(&args, params.release)
}

pub fn get_all_packages(
    include_examples: bool,
    include_testcases: bool,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut package_paths = vec![];
    for category in all_directories_in(Path::new("guest/rust/packages"))? {
        for package in all_directories_in(&category.path())? {
            package_paths.push(package.path());
        }
    }
    if include_examples {
        package_paths.append(&mut get_all_examples(include_testcases)?);
    }

    package_paths.sort();
    Ok(package_paths)
}

fn get_all_examples(include_testcases: bool) -> anyhow::Result<Vec<PathBuf>> {
    let mut examples = Vec::new();

    for guest in all_directories_in(Path::new("guest")).context("Failed to find guest directory")? {
        let examples_path = guest.path().join("examples");
        let dirs = match all_directories_in(&examples_path) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Failed to query examples directory at {examples_path:?}: {e}");
                continue;
            }
        };

        for category in dirs {
            for example in all_directories_in(&category.path())? {
                examples.push(example.path());
            }
        }

        if include_testcases {
            let testcases_path = guest.path().join("testcases");
            if testcases_path.exists() {
                for entry in all_directories_in(&testcases_path)? {
                    examples.push(entry.path());
                }
            }
        }
    }

    examples.sort_by_key(|path| path.clone());

    Ok(examples)
}
