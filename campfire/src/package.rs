use std::path::Path;

use clap::Parser;

use crate::util::{all_directories_in, run_ambient};

#[derive(Parser, Clone)]
#[clap(trailing_var_arg = true)]
pub enum Package {
    /// Build all standard packages
    BuildAll,
    /// Publish all standard packages
    DeployAll {
        #[arg(long)]
        token: String,
    },
}

pub fn main(args: &Package) -> anyhow::Result<()> {
    match args {
        Package::BuildAll => build_all(),
        Package::DeployAll { token } => deploy_all(token),
    }
}

pub fn build_all() -> anyhow::Result<()> {
    let package_paths = get_all_packages()?;

    for path in &package_paths {
        run_ambient(&["build", path.as_str(), "--clean-build"], true)?;
    }

    Ok(())
}

pub fn deploy_all(token: &str) -> anyhow::Result<()> {
    let package_paths = get_all_packages()?;

    let mut args = vec!["deploy"];
    for (idx, path) in package_paths.iter().enumerate() {
        if idx != 0 {
            args.push("--extra-packages");
        }
        args.push(path);
    }
    args.push("--token");
    args.push(token);
    args.push("--clean-build");

    run_ambient(&args, true)
}

fn get_all_packages() -> anyhow::Result<Vec<String>> {
    let mut package_paths = vec![];
    for category in all_directories_in(Path::new("guest/rust/packages"))? {
        for package in all_directories_in(&category.path())? {
            package_paths.push(package.path().to_string_lossy().to_string());
        }
    }
    package_paths.sort();
    Ok(package_paths)
}
