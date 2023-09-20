use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{ensure, Context};
use cargo_toml::Inheritable;
use clap::Parser;
use itertools::Itertools;
use serde::Deserialize;
use std::str;

use crate::package::get_all_packages;

#[derive(Parser, Clone)]
pub enum Release {
    /// Changes the Ambient version across all crates and documentation to match the given version
    UpdateVersion {
        #[arg()]
        new_version: String,

        #[arg(long, default_value_t)]
        /// If set, this will update everything *but* the `ambient_version` field of the packages.
        ///
        /// This is used to preserve the version of deployed packages in the CI.
        no_package_ambient_version_update: bool,
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
        no_crates_io_validity: bool,

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

pub fn main(args: &Release) -> anyhow::Result<()> {
    match args {
        Release::UpdateVersion {
            new_version,
            no_package_ambient_version_update,
        } => update_version(new_version, *no_package_ambient_version_update),
        Release::UpdateMsrv { new_version } => update_msrv(new_version),
        Release::Publish { execute } => publish(*execute),
        Release::Check {
            no_docker,
            no_crates_io_validity,
            no_msrv,
            no_build,
            no_changelog,
            no_readme,
        } => check_release(
            *no_docker,
            *no_crates_io_validity,
            *no_msrv,
            *no_build,
            *no_changelog,
            *no_readme,
        ),
    }
}

const DOCKERFILE: &str = "Dockerfile";
const AMBIENT_MANIFEST: &str = "schema/ambient.toml";
const AMBIENT_MANIFEST_INCLUDES: &str = "schema/schema";
const ROOT_CARGO: &str = "Cargo.toml";
const WEB_CARGO: &str = "web/Cargo.toml";
const GUEST_RUST_CARGO: &str = "guest/rust/Cargo.toml";
const ADVANCED_INSTALLING_DOCS: &str = "docs/src/reference/advanced_installing.md";
const CHANGELOG: &str = "CHANGELOG.md";
const README: &str = "README.md";
const INTRODUCTION: &str = "docs/src/introduction.md";

fn check_release(
    no_docker: bool,
    no_crates_io_validity: bool,
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

    // the crates can all be published to crates.io
    if !no_crates_io_validity {
        check_crates_io_validity()?;
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

fn update_version(
    new_version: &str,
    no_package_ambient_version_update: bool,
) -> anyhow::Result<()> {
    if !new_version.starts_with(char::is_numeric) {
        anyhow::bail!("version must start with an integer");
    }

    // This must be done first, before we mutate anything, to ensure that it's in a consistent state
    let all_publishable_crates = get_all_publishable_crates()?;

    if !dbg!(no_package_ambient_version_update) {
        fn add_ambient_version(toml: &mut toml_edit::Document, new_version: &str) {
            toml["package"]["ambient_version"] = toml_edit::value(new_version);
        }

        edit_toml(AMBIENT_MANIFEST, |toml| {
            add_ambient_version(toml, new_version);
        })?;

        for path in std::fs::read_dir(AMBIENT_MANIFEST_INCLUDES)?
            .filter_map(Result::ok)
            .map(|e| e.path())
        {
            edit_toml(path, |toml| {
                add_ambient_version(toml, new_version);
            })?;
        }

        for path in get_all_packages(true, true, true)? {
            edit_toml(path.join("ambient.toml"), |toml| {
                add_ambient_version(toml, new_version);
            })?;
        }
    }

    edit_toml(AMBIENT_MANIFEST, |toml| {
        toml["package"]["version"] = toml_edit::value(new_version);
    })?;

    for path in std::fs::read_dir(AMBIENT_MANIFEST_INCLUDES)?
        .filter_map(Result::ok)
        .map(|e| e.path())
    {
        edit_toml(path, |toml| {
            toml["package"]["version"] = toml_edit::value(new_version);
        })?;
    }

    edit_toml(ROOT_CARGO, |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);
    })?;

    edit_toml(WEB_CARGO, |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);
    })?;

    // Fix all of the dependency versions of publishable Ambient crates
    edit_toml(GUEST_RUST_CARGO, |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);
        update_ambient_dependency_versions(
            &all_publishable_crates,
            &mut toml["workspace"]["dependencies"],
            new_version,
        );
    })?;

    for (path, _) in &all_publishable_crates {
        edit_toml(path, |toml| {
            update_ambient_dependency_versions(
                &all_publishable_crates,
                &mut toml["dependencies"],
                new_version,
            );
        })?;
    }

    edit_file(ADVANCED_INSTALLING_DOCS, |document| {
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

fn update_ambient_dependency_versions(
    all_publishable_crates: &[(PathBuf, cargo_toml::Manifest)],
    dependencies: &mut toml_edit::Item,
    new_version: &str,
) {
    let candidate_crates = all_publishable_crates
        .iter()
        .map(|p| p.1.package.as_ref().unwrap().name.clone())
        .collect::<HashSet<_>>();

    for (key, value) in dependencies
        .as_table_like_mut()
        .expect("dependencies is not a table")
        .iter_mut()
    {
        if !candidate_crates.contains(key.get()) {
            continue;
        }

        let Some(table) = value.as_table_like_mut() else {
            continue;
        };
        if table.contains_key("workspace") {
            continue;
        }
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

    edit_file(ADVANCED_INSTALLING_DOCS, |document| {
        let begin = "<!-- rust-version-begin -->";
        let end = "<!-- rust-version-end -->";
        let begin_index = document.find(begin).expect("no begin") + begin.len();
        let end_index = document.find(end).expect("no end");

        let mut document = document.to_owned();
        document.replace_range(begin_index..end_index, new_version);
        document
    })?;

    Ok(())
}

fn publish(execute: bool) -> anyhow::Result<()> {
    let crates = get_all_publishable_crates()?;

    #[derive(Debug)]
    enum Task {
        Publish(PathBuf, Vec<String>),
        Wait(usize),
    }

    let tasks = crates
        .into_iter()
        .map(|p| {
            let crate_path = p.0.parent().unwrap().canonicalize().unwrap();
            let crates_path = crate_path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy();

            let features = if crates_path == "shared_crates" {
                if !p.1.features.contains_key("default") && p.1.features.contains_key("native") {
                    vec!["native".to_string()]
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

            Task::Publish(crate_path, features)
        })
        .chunks(5)
        .into_iter()
        .flat_map(|c| c.chain(std::iter::once(Task::Wait(30))))
        .collect_vec();
    // Remove the last wait
    let tasks = &tasks[0..tasks.len() - 1];

    match execute {
        true => {
            for task in tasks {
                match task {
                    Task::Publish(path, features) => {
                        let mut command = Command::new("cargo");
                        command.arg("publish");
                        command.arg("--no-verify");
                        if !features.is_empty() {
                            command.arg("-F").arg(features.join(","));
                        }

                        let status = command.current_dir(path).spawn()?.wait()?;
                        if !status.success() {
                            anyhow::bail!("failed to upload {}", path.display());
                        }
                    }
                    Task::Wait(seconds) => {
                        std::thread::sleep(std::time::Duration::from_secs((*seconds).try_into()?))
                    }
                }
            }
        }
        false => {
            for task in tasks {
                match task {
                    Task::Publish(path, features) => {
                        let features = features.join(",");
                        println!(
                            "cd {} && cargo publish --no-verify{}; cd -",
                            path.display(),
                            if features.is_empty() {
                                "".to_string()
                            } else {
                                format!(" -F {}", features)
                            }
                        )
                    }
                    Task::Wait(seconds) => println!("sleep {}", seconds),
                }
            }
        }
    }

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

fn edit_toml(
    path: impl AsRef<Path> + Clone,
    f: impl Fn(&mut toml_edit::Document),
) -> anyhow::Result<()> {
    edit_file(path.clone(), |input| {
        let mut toml = input.parse::<toml_edit::Document>().unwrap();
        f(&mut toml);
        toml.to_string()
    })
    .with_context(|| format!("Failed to edit file {:?}", path.as_ref()))
}

fn check_docker_build() -> anyhow::Result<()> {
    log::info!("Building Docker image...");
    let success = Command::new("docker")
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
    let success = Command::new("docker")
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

fn check_crates_io_validity() -> anyhow::Result<()> {
    let crates = get_all_publishable_crates()?;
    for (path, manifest) in crates {
        let Some(package) = manifest.package else {
            anyhow::bail!("no package for {}", path.display())
        };

        anyhow::ensure!(
            non_empty_inheritable_string(&package.license),
            "no license in {}",
            path.display()
        );

        anyhow::ensure!(
            non_empty_inheritable_string(&package.description),
            "no description in {}",
            path.display()
        );

        anyhow::ensure!(
            non_empty_inheritable_string(&package.repository),
            "no repository in {}",
            path.display()
        );

        let parent_path = path.parent().unwrap();
        anyhow::ensure!(
            parent_path.join("README.md").is_file(),
            "no README.md in {}",
            parent_path.display()
        );
    }

    fn non_empty_inheritable_string(s: &Option<Inheritable<String>>) -> bool {
        s.as_ref()
            .and_then(|s| s.get().ok())
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }

    Ok(())
}

fn check_msrv() -> anyhow::Result<()> {
    log::info!("Checking MSRV...");

    let msrv = {
        let output = Command::new("cargo")
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
    let success = Command::new("cargo")
        .args(["build", "--release"])
        .spawn()?
        .wait()?
        .success();
    if !success {
        anyhow::bail!("failed to build root crate");
    }

    let success = Command::new("cargo")
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
    let mut command = Command::new("cargo");
    command.current_dir(path);
    command.args(["check"]);

    if !command.spawn()?.wait()?.success() {
        anyhow::bail!("Failed to check Rust code at {}", path.display());
    }

    Ok(())
}

fn get_all_publishable_crates() -> anyhow::Result<Vec<(PathBuf, cargo_toml::Manifest)>> {
    // Our publishing process is complicated by the presence of two workspaces
    // that share crates. None of the existing tooling, as far as I can tell,
    // handles this well.
    //
    // To deal with this, we use `guppy` to construct a graph for each workspace,
    // and then we fuse them together to produce the final publish list in topological
    // order.

    use guppy::graph::DependencyDirection;

    let mut manifests = Manifests::default();

    let ambient_crates = {
        let ambient_graph = guppy::MetadataCommand::new()
            .manifest_path(ROOT_CARGO)
            .build_graph()?;
        let ambient_id = ambient_graph
            .resolve_package_name("ambient")
            .package_ids(DependencyDirection::Forward)
            .next()
            .unwrap();
        let ambient_wasm_id = ambient_graph
            .resolve_package_name("ambient_wasm")
            .package_ids(DependencyDirection::Forward)
            .next()
            .unwrap();

        let mut ambient_crates = ambient_graph
            .query_forward([ambient_id])?
            .resolve()
            .package_ids(DependencyDirection::Forward)
            .filter(|id| {
                // We purposely exclude the WASM crate, as well as anything
                // that depends on it, as it has Git dependencies that we
                // can't publish at present.
                !ambient_graph
                    .depends_on(id, ambient_wasm_id)
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        ambient_crates.reverse();

        ambient_crates
            .iter()
            .map(|p| p.repr().split_ascii_whitespace().next().unwrap())
            .filter(|p| manifests.exists(p))
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
    };

    let api_crates = {
        let api_graph = guppy::MetadataCommand::new()
            .manifest_path(GUEST_RUST_CARGO)
            .build_graph()?;
        let api_id = api_graph
            .resolve_package_name("ambient_api")
            .package_ids(DependencyDirection::Forward)
            .next()
            .unwrap();

        let mut api_crates = api_graph
            .query_forward([api_id])?
            .resolve()
            .package_ids(DependencyDirection::Forward)
            .collect::<Vec<_>>();
        api_crates.reverse();

        api_crates
            .iter()
            .map(|p| p.repr().split_ascii_whitespace().next().unwrap())
            .filter(|p| manifests.exists(p))
            .filter(|p| !ambient_crates.iter().any(|tp| tp == p))
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
    };

    Ok(ambient_crates
        .into_iter()
        .chain(api_crates)
        .map(|p| manifests.get(&p).unwrap())
        .collect_vec())
}

#[derive(Default)]
/// Helper that caches manifests for faster lookup so that we can easily
/// determine if a particular package actually belongs to Ambient
struct Manifests {
    cache: HashMap<PathBuf, cargo_toml::Manifest>,
}
impl Manifests {
    fn get(&mut self, name: &str) -> Option<(PathBuf, cargo_toml::Manifest)> {
        let folder_name = if let Some(stripped) = name.strip_prefix("ambient") {
            stripped.strip_prefix('_').unwrap_or(stripped)
        } else {
            name
        };

        [
            Path::new("crates").join(folder_name).join("Cargo.toml"),
            Path::new("libs").join(folder_name).join("Cargo.toml"),
            Path::new("shared_crates")
                .join(folder_name)
                .join("Cargo.toml"),
            "guest/rust/api/Cargo.toml".into(),
            "guest/rust/api_core/api_macros/Cargo.toml".into(),
            "guest/rust/api_core/Cargo.toml".into(),
            "app/Cargo.toml".into(),
        ]
        .into_iter()
        .filter(|p| p.exists())
        .find_map(|p| {
            let manifest = self.cache.entry(p.clone()).or_insert_with(|| {
                // Intentionally manually read the file as we do not want to
                // use `cargo_toml`'s dependency resolution.
                cargo_toml::Manifest::from_str(&std::fs::read_to_string(&p).unwrap())
                    .unwrap_or_else(|_| panic!("failed to parse {:?}", p))
            });

            if manifest.package().name == name {
                Some((p, manifest.clone()))
            } else {
                None
            }
        })
    }

    fn exists(&mut self, name: &str) -> bool {
        self.get(name).is_some()
    }
}
