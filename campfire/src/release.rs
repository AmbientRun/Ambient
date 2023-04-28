use std::path::Path;

use clap::Parser;
use std::path::PathBuf;
use markdown_extract::extract_from_path;
use regex::Regex;
use anyhow::{ensure, bail};
use std::str;


#[derive(Parser, Clone)]
pub enum Release {
    /// Changes the Ambient version across all crates and documentation to match the given version
    UpdateVersion {
        #[clap()]
        new_version: String,
    },
    /// Changes the minimum supported Rust version across all crates and documentation to match the given version
    UpdateMsrv {
        #[clap()]
        new_version: String,
    },

    /// Validates release artifacts
    Check {},
}

pub(crate) fn main(args: &Release) -> anyhow::Result<()> {
    match args {
        Release::UpdateVersion { new_version } => update_version(new_version),
        Release::UpdateMsrv { new_version } => update_msrv(new_version),
        Release::Check {} => check_release(),
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

fn check_release() -> anyhow::Result<()>{
    // https://github.com/AmbientRun/Ambient/issues/314
    // the Dockerfile can run an Ambient server
    check_docker_build()?;
    check_docker_run()?;

    // the MSRV is correct for both the host and the API
    check_msrv()?;

    // both the runtime and the guest can build with no errors
    check_builds()?;

    // the CHANGELOG's unreleased section is empty
    check_changelog()?;

    // README.md and docs/src/introduction.md match their introductory text
    check_readme()?;

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
    std::process::Command::new("docker")
        .args(["build", ".", "-t", "ambient_campfire"])
        .spawn()?
        .wait()?;

    Ok(())
}

fn check_docker_run() -> anyhow::Result<()> {
    std::process::Command::new("docker")
        .args(["run", "--rm", "-d", "ambient_campfire"])
        .spawn()?
        .wait()?;

    Ok(())
}


fn check_msrv() -> anyhow::Result<()> {
    println!("checking MSRV...");
    let msrv_out = std::process::Command::new("cargo")
        .args(["msrv", "--output-format", "minimal"])
        .output()?;

    let msrv_str = str::from_utf8(&msrv_out.stdout).unwrap().trim();

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
            rust_version == msrv_str,
            "{} does not match MSRV: expected {}, found {}",
            cargo_file,
            msrv_str,
            rust_version
        );
    }

    println!("MSRV OK");
    Ok(())
}


fn check_builds() -> anyhow::Result<()> {
    println!("checking Builds...");
    std::process::Command::new("cargo")
        .args(["build", "--release"])
        .spawn()?
        .wait()?;

    std::process::Command::new("cargo")
        .args(["build", "--release", "--target-dir", "guest/rust"])
        .spawn()?
        .wait()?;

    println!("Builds Ok");
    Ok(())
}

fn check_changelog() -> anyhow::Result<()>{
    println!("checking CHANGELOG...");
    let header_regex = Regex::new(r"\bunreleased\b")?;
    let unreleased_content = extract_from_path(&PathBuf::from(CHANGELOG), &header_regex)?;
    match unreleased_content.is_empty() {
        false => {
          bail!("Unreleased content in CHANGELOG.md");
        },
        true => {
            println!("CHANGELOG OK");
            Ok(())
        }
    }
}

fn check_readme() -> anyhow::Result<()> {
    println!("checking README Intro....");
    let intro = std::fs::read_to_string(INTRODUCTION)?
        .lines()
        .skip(1) // Skip the first line: # Introduction // not in the README
        .collect::<Vec<&str>>()
        .join("\n");

    let readme = std::fs::read_to_string(README)?;

    ensure!(readme.contains(&intro), "README intro content does not match!");

    println!("README intro OK");
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
