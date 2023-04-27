use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;
use petgraph::visit::EdgeRef;

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
    /// Publish the API and required crates to crates.io. This is done automatically on release.
    /// Dry run by default.
    Publish {
        #[clap(long)]
        execute: bool,
    },
}

pub(crate) fn main(args: &Release) -> anyhow::Result<()> {
    match args {
        Release::UpdateVersion { new_version } => update_version(new_version),
        Release::UpdateMsrv { new_version } => update_msrv(new_version),
        Release::Publish { execute } => publish(*execute),
    }
}

const DOCKERFILE: &str = "Dockerfile";
const AMBIENT_MANIFEST: &str = "shared_crates/schema/src/ambient.toml";
const ROOT_CARGO: &str = "Cargo.toml";
const WEB_CARGO: &str = "web/Cargo.toml";
const GUEST_RUST_CARGO: &str = "guest/rust/Cargo.toml";
const INSTALLING_DOCS: &str = "docs/src/user/installing.md";

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

    use cargo_lock::Dependency;

    let mut graph = petgraph::Graph::<Dependency, ()>::new();
    let mut nodes_by_name = HashMap::<String, petgraph::graph::NodeIndex>::new();
    {
        let mut nodes = HashMap::<Dependency, petgraph::graph::NodeIndex>::new();
        let api_lockfile = cargo_lock::Lockfile::load("guest/rust/Cargo.lock")?;
        let root_lockfile = cargo_lock::Lockfile::load("Cargo.lock")?;

        let mut populate_graph = |lockfile: &cargo_lock::Lockfile| {
            for package in &lockfile.packages {
                let node_index = graph.add_node(package.into());
                nodes.insert(package.into(), node_index);
                nodes_by_name.insert(package.name.to_string(), node_index);
            }

            for package in &lockfile.packages {
                let Some(&parent_index) = nodes.get(&package.into()) else { continue; };
                for dependency in &package.dependencies {
                    if let Some(&child_index) = nodes.get(&dependency) {
                        graph.add_edge(parent_index, child_index, ());
                    }
                }
            }
        };

        populate_graph(&api_lockfile);
        populate_graph(&root_lockfile);
    }
    let nodes_by_name = nodes_by_name;

    fn find_crate_manifest(name: &str) -> Option<(PathBuf, cargo_toml::Manifest)> {
        let stripped = name.strip_prefix("ambient_")?;
        [
            Path::new("crates").join(stripped).join("Cargo.toml"),
            Path::new("libs").join(stripped).join("Cargo.toml"),
            Path::new("shared_crates").join(stripped).join("Cargo.toml"),
            "guest/rust/api/Cargo.toml".into(),
            "guest/rust/api_core/api_macros/Cargo.toml".into(),
            "guest/rust/api_core/Cargo.toml".into(),
        ]
        .into_iter()
        .filter(|p| p.exists())
        .find_map(|p| {
            let toml_contents = std::fs::read_to_string(&p).unwrap();
            let manifest = &cargo_toml::Manifest::from_str(&toml_contents).unwrap();

            if manifest.package().name == name {
                Some((p, manifest.clone()))
            } else {
                None
            }
        })
    }

    let ambient_crates: HashMap<String, (PathBuf, cargo_toml::Manifest)> = nodes_by_name
        .keys()
        .filter(|k| k.starts_with("ambient_"))
        .cloned()
        .filter_map(|k| Some((k.clone(), find_crate_manifest(&k)?)))
        .collect();

    graph.retain_edges(|g, e| {
        let (source, target) = g.edge_endpoints(e).unwrap();
        let source_name = g.node_weight(source).unwrap().name.to_string();
        let target_name = g.node_weight(target).unwrap().name.to_string();

        if source_name.starts_with("ambient_") && target_name.starts_with("ambient_") {
            if let Some((_, manifest)) = ambient_crates.get(&source_name) {
                if manifest.dev_dependencies.keys().any(|d| *d == target_name) {
                    return false;
                }
            }
        }

        true
    });

    let mut bfs = petgraph::visit::Bfs::new(&graph, nodes_by_name["ambient_api"]);
    let mut names = vec![];
    while let Some(nx) = bfs.next(&graph) {
        let name = graph.node_weight(nx).unwrap().name.as_str();
        if !ambient_crates.contains_key(name) {
            continue;
        }
        names.push(name);
    }
    names.reverse();
    dbg!(names);

    let mut subgraph = petgraph::Graph::<Dependency, ()>::new();
    petgraph::visit::depth_first_search(&graph, [nodes_by_name["ambient_api"]], |event| {
        use petgraph::visit::DfsEvent;

        match event {
            DfsEvent::TreeEdge(n1, n2)
            | DfsEvent::BackEdge(n1, n2)
            | DfsEvent::CrossForwardEdge(n1, n2) => {
                subgraph.add_edge(n1, n2, ());
            }
            DfsEvent::Discover(_, _) | DfsEvent::Finish(_, _) => {}
        }
    });

    let toposort: Vec<String> = petgraph::algo::toposort(&subgraph, None)
        .map_err(|c| anyhow::anyhow!("{c:?}"))?
        .into_iter()
        .filter_map(|n| {
            let dependency = graph.node_weight(n)?;
            Some(format!("{} {}", dependency.name, dependency.version))
        })
        .filter(|n| n.starts_with("ambient_"))
        .collect();
    dbg!(toposort);

    // dbg!(graph
    //     .edges_directed(
    //         nodes_by_name["ambient_element"],
    //         petgraph::Direction::Outgoing
    //     )
    //     .map(|e| (
    //         graph.node_weight(e.source()).unwrap().name.as_str(),
    //         graph.node_weight(e.target()).unwrap().name.as_str()
    //     ))
    //     .collect::<Vec<_>>());

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
