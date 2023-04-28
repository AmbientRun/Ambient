use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Context};
use clap::Parser;
use serde::Deserialize;
use std::str;

#[derive(Parser, Clone)]
pub enum Release {
    /// Changes the Ambient version across all crates and documentation to match the given version
    UpdateVersion {
        #[arg()]
        new_version: String,
    },
    /// Changes the minimum supported Rust version across all crates and documentation to match the given version
    UpdateMsrv {
        #[arg()]
        new_version: String,
    },
    /// Publish the API and required crates to crates.io. This is done automatically on release.
    /// Dry run by default.
    Publish {
        #[clap(long)]
        execute: bool,
    },
    /// Checks that Ambient is ready for a release to be cut
    Check {
        #[arg(long)]
        no_docker: bool,

        #[arg(long)]
        no_msrv: bool,

        #[arg(long)]
        no_build: bool,

        #[arg(long)]
        no_changelog: bool,

        #[arg(long)]
        no_readme: bool,
    },
}

pub(crate) fn main(args: &Release) -> anyhow::Result<()> {
    match args {
        Release::UpdateVersion { new_version } => update_version(new_version),
        Release::UpdateMsrv { new_version } => update_msrv(new_version),
        Release::Publish { execute } => publish(*execute),
        Release::Check {
            no_docker,
            no_msrv,
            no_build,
            no_changelog,
            no_readme,
        } => check_release(*no_docker, *no_msrv, *no_build, *no_changelog, *no_readme),
    }
}

const DOCKERFILE: &str = "Dockerfile";
const AMBIENT_MANIFEST: &str = "shared_crates/schema/src/ambient.toml";
const ROOT_CARGO: &str = "Cargo.toml";
const WEB_CARGO: &str = "web/Cargo.toml";
const GUEST_RUST_CARGO: &str = "guest/rust/Cargo.toml";
const INSTALLING_DOCS: &str = "docs/src/user/installing.md";
const CHANGELOG: &str = "CHANGELOG.md";
const README: &str = "README.md";
const INTRODUCTION: &str = "docs/src/introduction.md";

fn check_release(
    no_docker: bool,
    no_msrv: bool,
    no_build: bool,
    no_changelog: bool,
    no_readme: bool,
) -> anyhow::Result<()> {
    // https://github.com/AmbientRun/Ambient/issues/314
    // the Dockerfile can run an Ambient server
    if !no_docker {
        check_docker_build()?;
        check_docker_run()?;
    }

    // the MSRV is correct for both the host and the API
    if !no_msrv {
        check_msrv()?;
    }

    // both the runtime and the guest can build with no errors
    if !no_build {
        check_builds()?;
    }

    // the CHANGELOG's unreleased section is empty
    if !no_changelog {
        check_changelog()?;
    }

    // README.md and docs/src/introduction.md match their introductory text
    if !no_readme {
        check_readme()?;
    }

    Ok(())
}

fn update_version(new_version: &str) -> anyhow::Result<()> {
    if !new_version.starts_with(char::is_numeric) {
        anyhow::bail!("version must start with an integer");
    }

    edit_toml(AMBIENT_MANIFEST, |toml| {
        toml["project"]["version"] = toml_edit::value(new_version);
    })?;

    edit_toml(ROOT_CARGO, |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);
    })?;

    edit_toml(WEB_CARGO, |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);
    })?;

    edit_toml(GUEST_RUST_CARGO, |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);
        update_ambient_dependency_versions(&mut toml["workspace"]["dependencies"], new_version);
    })?;

    // Fix all of the dependency versions for Ambient crates
    for path in ["libs", "shared_crates"] {
        for dir in std::fs::read_dir(path)?
            .filter_map(Result::ok)
            .map(|de| de.path())
            .filter(|p| p.is_dir())
        {
            edit_toml(dir.join("Cargo.toml"), |toml| {
                update_ambient_dependency_versions(&mut toml["dependencies"], new_version);
            })?;
        }
    }

    edit_file(INSTALLING_DOCS, |document| {
        const PREFIX: &str = "cargo install --git https://github.com/AmbientRun/Ambient.git --tag";
        document
            .lines()
            .map(|l| {
                if l.starts_with(PREFIX) {
                    format!("{PREFIX} v{new_version} ambient")
                } else {
                    l.to_string()
                }
            })
            // newline at the end
            .chain(std::iter::once("".to_string()))
            .collect::<Vec<String>>()
            .join("\n")
    })?;

    // Run `cargo check` in the root and API to force the lockfile to update
    check(".")?;
    check("guest/rust")?;

    Ok(())
}

fn update_ambient_dependency_versions(dependencies: &mut toml_edit::Item, new_version: &str) {
    for (key, value) in dependencies
        .as_table_like_mut()
        .expect("dependencies is not a table")
        .iter_mut()
    {
        if !key.starts_with("ambient_") {
            continue;
        }

        let Some(table) = value.as_table_like_mut() else { continue; };
        table.insert("version", toml_edit::value(new_version));
    }
}

fn update_msrv(new_version: &str) -> anyhow::Result<()> {
    edit_toml(ROOT_CARGO, |toml| {
        toml["workspace"]["package"]["rust-version"] = toml_edit::value(new_version);
    })?;

    edit_toml(WEB_CARGO, |toml| {
        toml["workspace"]["package"]["rust-version"] = toml_edit::value(new_version);
    })?;

    edit_toml(GUEST_RUST_CARGO, |toml| {
        toml["workspace"]["package"]["rust-version"] = toml_edit::value(new_version);
    })?;

    edit_file(DOCKERFILE, |document| {
        const PREFIX: &str = "FROM rust:";
        document
            .lines()
            .map(|l| {
                if l.starts_with(PREFIX) {
                    format!("{PREFIX}{new_version}")
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    edit_file(INSTALLING_DOCS, |document| {
        let begin = "<!-- rust-version-begin !-->";
        let end = "<!-- rust-version-end !-->";
        let begin_index = document.find(begin).expect("no begin") + begin.len();
        let end_index = document.find(end).expect("no end");

        let mut document = document.to_owned();
        document.replace_range(begin_index..end_index, new_version);
        document
    })?;

    Ok(())
}

fn publish(execute: bool) -> anyhow::Result<()> {
    // Our publishing process is complicated by the presence of two workspaces
    // that share crates. None of the existing tooling, as far as I can tell,
    // handles this well.
    //
    // To deal with this, this constructs a graph of the dependencies between
    // crates, and then publishes them in the correct order. These dependencies
    // are resolved across *both* workspaces using their `Cargo.lock`s.
    //
    // However, this is complicated by the presence of cycles in the dependency
    // graph as a result of dev-dependencies for testing. To work around this,
    // we parse through all of the manifests, locate the dev-dependencies,
    // and delete their corresponding edges.
    //
    // Once this is done, we topologically sort the graph and publish in that order.

    use guppy::graph::DependencyDirection;

    let graph = guppy::MetadataCommand::new()
        .manifest_path(ROOT_CARGO)
        .build_graph()?;
    let package_id = graph
        .resolve_package_name("ambient")
        .package_ids(DependencyDirection::Forward)
        .next()
        .unwrap();
    let mut packages = graph
        .query_forward([package_id])?
        .resolve()
        .package_ids(DependencyDirection::Forward)
        .collect::<Vec<_>>();
    packages.reverse();

    #[derive(Default)]
    struct Manifests {
        cache: HashMap<PathBuf, cargo_toml::Manifest>,
    }
    impl Manifests {
        fn exists(&mut self, name: &str) -> bool {
            let Some(stripped) = name.strip_prefix("ambient") else { return false; };
            let stripped = stripped.strip_prefix("_").unwrap_or(name);

            [
                Path::new("crates").join(stripped).join("Cargo.toml"),
                Path::new("libs").join(stripped).join("Cargo.toml"),
                Path::new("shared_crates").join(stripped).join("Cargo.toml"),
                "guest/rust/api/Cargo.toml".into(),
                "guest/rust/api_core/api_macros/Cargo.toml".into(),
                "guest/rust/api_core/Cargo.toml".into(),
                "app/Cargo.toml".into(),
            ]
            .into_iter()
            .filter(|p| p.exists())
            .any(|p| {
                let manifest = self.cache.entry(p.clone()).or_insert_with(|| {
                    // Intentionally manually read the file as we do not want to
                    // use `cargo_toml`'s dependency resolution.
                    cargo_toml::Manifest::from_str(&std::fs::read_to_string(&p).unwrap())
                        .expect(&format!("failed to parse {:?}", p))
                });

                manifest.package().name == name
            })
        }
    }
    let mut manifests = Manifests::default();

    let packages = packages
        .iter()
        .map(|p| p.repr().split_ascii_whitespace().next().unwrap())
        .filter(|p| manifests.exists(p))
        .collect::<Vec<_>>();

    dbg!(packages);

    Ok(())
}

fn edit_file(path: impl AsRef<Path>, f: impl Fn(&str) -> String) -> anyhow::Result<()> {
    let path = path.as_ref();
    let input = std::fs::read_to_string(path)?;
    let output = f(&input);
    // Only write the output if the difference is more than trailing newline
    if input.trim() != output.trim() {
        std::fs::write(path, output)?;
    }

    Ok(())
}

fn edit_toml(path: impl AsRef<Path>, f: impl Fn(&mut toml_edit::Document)) -> anyhow::Result<()> {
    edit_file(path, |input| {
        let mut toml = input.parse::<toml_edit::Document>().unwrap();
        f(&mut toml);
        toml.to_string()
    })
}

fn check_docker_build() -> anyhow::Result<()> {
    log::info!("Building Docker image...");
    let success = std::process::Command::new("docker")
        .args(["build", ".", "-t", "ambient_campfire"])
        .spawn()?
        .wait()?
        .success();
    if !success {
        anyhow::bail!("failed to build Docker image");
    }
    log::info!("Built Docker image.");

    Ok(())
}

fn check_docker_run() -> anyhow::Result<()> {
    log::info!("Running Docker instance...");
    let success = std::process::Command::new("docker")
        .args([
            "run",
            "--rm",
            "-it",
            "-v",
            &format!(
                "{}:/app",
                std::env::current_dir()?.to_string_lossy().as_ref(),
            ),
            "ambient_campfire",
            "cargo",
            "run",
            "--",
            "--help",
        ])
        .spawn()?
        .wait()?
        .success();
    if !success {
        anyhow::bail!("failed to execute cargo run in Docker instance");
    }
    log::info!("Ran Docker instance.");

    Ok(())
}

fn check_msrv() -> anyhow::Result<()> {
    log::info!("Checking MSRV...");

    let msrv = {
        let output = std::process::Command::new("cargo")
            .args([
                "msrv",
                "--output-format",
                "json",
                "--min",
                "1.60.0",
                "--include-all-patch-releases",
            ])
            .output()?;
        if !output.status.success() {
            anyhow::bail!("failed to execute cargo msrv");
        }

        let msrv_out = String::from_utf8(output.stdout)?;
        let last_line = msrv_out
            .lines()
            .last()
            .ok_or_else(|| anyhow::anyhow!("cargo msrv output is empty"))?;

        #[derive(Deserialize)]
        struct MsrvOutput {
            msrv: String,
            success: bool,
        }

        let output = serde_json::from_str::<MsrvOutput>(last_line)
            .context("could not parse cargo msrv output")?;

        if !output.success {
            anyhow::bail!("cargo msrv reported failure");
        }
        output.msrv
    };

    let cargo_files = [ROOT_CARGO, WEB_CARGO, GUEST_RUST_CARGO];
    for cargo_file in &cargo_files {
        let cargo_toml = std::fs::read_to_string(cargo_file)?;
        let cargo_toml_parsed = cargo_toml.parse::<toml::Value>()?;

        let rust_version = cargo_toml_parsed
            .get("workspace")
            .and_then(|w| w.get("package"))
            .and_then(|p| p.get("rust-version"))
            .and_then(|rv| rv.as_str())
            .ok_or_else(|| anyhow::anyhow!("Could not find rust-version in {}", cargo_file))?;

        ensure!(
            rust_version == msrv,
            "{} does not match MSRV: expected {}, found {}",
            cargo_file,
            msrv,
            rust_version
        );
    }

    // TODO: check dockerfile

    log::info!("MSRV OK.");
    Ok(())
}

fn check_builds() -> anyhow::Result<()> {
    log::info!("Checking builds...");
    let success = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .spawn()?
        .wait()?
        .success();
    if !success {
        anyhow::bail!("failed to build root crate");
    }

    let success = std::process::Command::new("cargo")
        .current_dir("guest/rust")
        .args(["build", "--release"])
        .spawn()?
        .wait()?
        .success();
    if !success {
        anyhow::bail!("failed to build guest crate");
    }

    log::info!("Builds OK.");
    Ok(())
}

fn check_changelog() -> anyhow::Result<()> {
    log::info!("Checking CHANGELOG...");

    // TODO: Currently unimplemented; the implementation needs to handle
    // commented out Markdown, so it has to have some degree of smarts about it
    let _changelog = std::fs::read_to_string(CHANGELOG)?;

    log::info!("CHANGELOG skipped (unimplemented, see code).");
    Ok(())
}

fn check_readme() -> anyhow::Result<()> {
    log::info!("Checking README intro...");
    let intro = std::fs::read_to_string(INTRODUCTION)?
        .lines()
        .skip(1) // Skip the first line: # Introduction // not in the README
        .collect::<Vec<&str>>()
        .join("\n");

    let readme = std::fs::read_to_string(README)?;

    ensure!(
        readme.contains(&intro),
        "README intro content does not match!"
    );

    log::info!("README intro OK.");
    Ok(())
}

fn check(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    let mut command = std::process::Command::new("cargo");
    command.current_dir(path);
    command.args(&["check"]);

    if !command.spawn()?.wait()?.success() {
        anyhow::bail!("Failed to check Rust code at {}", path.display());
    }

    Ok(())
}
