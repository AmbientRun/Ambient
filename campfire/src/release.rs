use std::path::Path;

use clap::Parser;

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

fn check_release() -> anyhow::Result<()>{
    // https://github.com/AmbientRun/Ambient/issues/314
    // the Dockerfile can run an Ambient server
    // the MSRV is correct for both the host and the API
    // both the runtime and the guest can build with no errors
    // the CHANGELOG's unreleased section is empty
    // README.md and docs/src/introduction.md match their introductory text

    run_docker_build()?
    check_docker_server()?
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

fn run_docker_build() -> anyhow::Result<()> {
    std::process::Command::new("docker")
        .args(["build", "-t", "ambient_check"])
        .spawn()?
        .wait()?;

    Ok(())
}

fn check_docker_server() -> anyhow::Result<()> {
    std::process::Command::new("docker")
        .args(["run", "--rm", "-it", "-e", "bash", "-v", "./:/app ambient"])
        .spawn()?
        .wait()?;

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
