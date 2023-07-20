use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;

#[derive(Parser, Clone)]
#[clap(trailing_var_arg = true)]
pub enum Example {
    /// Clean all example build artifacts
    Clean,
    /// Run an example
    Run(Run),
    /// Run all the examples in order
    RunAll {
        /// Whether or not to run Ambient in release mode
        #[arg(short, long, default_value_t = false)]
        release: bool,
        /// The args to pass through to `ambient`
        args: Vec<String>,
    },
    /// Check all the examples
    CheckAll,
}

#[derive(Parser, Clone)]
#[clap(trailing_var_arg = true)]
/// Run an example
pub struct Run {
    /// The name of the example to run
    pub example: String,
    /// Whether or not to run Ambient in release mode
    #[arg(short, long, default_value_t = false)]
    pub release: bool,
    /// The args to pass through to `ambient`
    pub args: Vec<String>,
}

pub(crate) fn main(args: &Example) -> anyhow::Result<()> {
    match args {
        Example::Clean => clean(),
        Example::Run(args) => run(args),
        Example::RunAll { release, args } => run_all(*release, args),
        Example::CheckAll => check_all(),
    }
}

pub(crate) fn clean() -> anyhow::Result<()> {
    log::info!("Cleaning examples...");
    for (example_path, _) in all_examples(true)? {
        let build_path = example_path.join("build");
        if !build_path.exists() {
            continue;
        }

        std::fs::remove_dir_all(&build_path)?;
        log::info!("Removed build directory for {}.", example_path.display());
    }
    log::info!("Done cleaning examples.");
    Ok(())
}

pub(crate) fn run(args: &Run) -> anyhow::Result<()> {
    let Run {
        example,
        release,
        args,
    } = args;

    let (example_path, _) = all_examples(true)?
        .into_iter()
        .find(|(p, _)| p.ends_with(example))
        .ok_or_else(|| anyhow::anyhow!("no example found with name {}", example))?;

    log::info!(
        "Running example {} (Ambient built with release: {release}, extra args {args:?})...",
        example_path.display()
    );
    run_project(&example_path, *release, args)
}

fn run_all(release: bool, args: &[String]) -> anyhow::Result<()> {
    for (example_path, _) in all_examples(true)? {
        log::info!(
            "Running example {} (Ambient built with release: {release}, extra args {args:?})...",
            example_path.display()
        );
        run_project(&example_path, release, args)?;
    }

    Ok(())
}

fn check_all() -> anyhow::Result<()> {
    // Rust
    {
        let root_path = Path::new("guest/rust");
        log::info!("Checking Rust examples...");

        for features in ["", "client", "server", "client,server"] {
            log::info!("Checking Rust examples with features `{}`...", features);

            let mut command = std::process::Command::new("cargo");
            command.current_dir(root_path);
            command.args(["clippy"]);
            command.env("RUSTFLAGS", "-Dwarnings");

            if !features.is_empty() {
                command.args(["--features", features]);
            }

            if !command.spawn()?.wait()?.success() {
                anyhow::bail!("Failed to check Rust examples with features {}.", features);
            }
        }

        log::info!("Checked Rust examples.");
    }

    Ok(())
}

fn run_project(project: &Path, ambient_release: bool, extra_args: &[String]) -> anyhow::Result<()> {
    let mut args = vec!["run"];
    let project = project.to_string_lossy();
    args.push(&project);
    if !extra_args.is_empty() {
        args.extend(extra_args.iter().map(|s| s.as_str()));
    }
    run_ambient(&args, ambient_release)
}

fn run_ambient(args: &[&str], release: bool) -> anyhow::Result<()> {
    // TODO: consider running other versions of Ambient
    let mut command = std::process::Command::new("cargo");
    command.arg("run");
    if release {
        command.arg("--release");
    }
    command.args(["-p", "ambient"]).args(args).spawn()?.wait()?;

    Ok(())
}

pub(crate) fn all_examples(with_testcases: bool) -> anyhow::Result<Vec<(PathBuf, String)>> {
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
                examples.push((
                    example.path(),
                    format!(
                        "{}/{}",
                        category.file_name().to_str().unwrap(),
                        example.file_name().to_str().unwrap()
                    ),
                ));
            }
        }

        if with_testcases {
            let testcases_path = guest.path().join("testcases");
            if testcases_path.exists() {
                for entry in all_directories_in(&testcases_path)? {
                    examples.push((
                        entry.path(),
                        format!("{}", entry.file_name().to_str().unwrap()),
                    ));
                }
            }
        }
    }

    Ok(examples)
}

fn all_directories_in(path: &Path) -> anyhow::Result<impl Iterator<Item = DirEntry>> {
    Ok(std::fs::read_dir(path)?
        .filter_map(Result::ok)
        .filter(|p| p.path().is_dir()))
}
