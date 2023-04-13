use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser, Clone)]
pub enum Example {
    /// Clean all example build artifacts
    Clean,
    /// Run an example
    Run {
        /// The name of the example to run
        example: String,
    },
    /// Run all the examples in order
    RunAll,
    /// Check all the examples
    CheckAll,
}

pub(crate) fn main(ex: &Example) -> anyhow::Result<()> {
    match ex {
        Example::Clean => clean(),
        Example::Run { example } => run(&example),
        Example::RunAll => run_all(),
        Example::CheckAll => check_all(),
    }
}

fn clean() -> anyhow::Result<()> {
    log::info!("Cleaning examples...");
    for example_path in all_examples()? {
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

fn run(name: &str) -> anyhow::Result<()> {
    let example_path = all_examples()?
        .into_iter()
        .find(|p| p.ends_with(name))
        .ok_or_else(|| anyhow::anyhow!("no example found with name {}", name))?;

    log::info!("Running example {}...", example_path.display());
    run_project(&example_path)
}

fn run_all() -> anyhow::Result<()> {
    for example_path in all_examples()? {
        log::info!("Running example {}...", example_path.display());
        run_project(&example_path)?;
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
            command.args(&["clippy"]);
            command.env("RUSTFLAGS", "-Dwarnings");

            if !features.is_empty() {
                command.args(&["--features", features]);
            }

            if !command.spawn()?.wait()?.success() {
                anyhow::bail!("Failed to check Rust examples with features {}.", features);
            }
        }

        log::info!("Checked Rust examples.");
    }

    Ok(())
}

fn run_project(project: &Path) -> anyhow::Result<()> {
    run_ambient(&["run", project.to_string_lossy().as_ref()])
}

fn run_ambient(args: &[&str]) -> anyhow::Result<()> {
    // TODO: consider running other versions of Ambient
    std::process::Command::new("cargo")
        .args(&["run", "-p", "ambient"])
        .args(args)
        .spawn()?
        .wait()?;

    Ok(())
}

fn all_examples() -> anyhow::Result<Vec<PathBuf>> {
    let mut examples = Vec::new();

    for guest in all_directories_in(Path::new("guest"))? {
        for category_path in all_directories_in(&guest.join("examples"))? {
            for example_path in all_directories_in(&category_path)? {
                examples.push(example_path);
            }
        }
    }

    Ok(examples)
}

fn all_directories_in(path: &Path) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    Ok(std::fs::read_dir(path)?
        .into_iter()
        .filter_map(Result::ok)
        .map(|de| de.path())
        .filter(|p| p.is_dir()))
}
