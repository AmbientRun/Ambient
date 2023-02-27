use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    download_asset::AssetsCacheOnDisk,
};
use clap::Parser;

mod cli;
mod client;
mod server;
mod shared;

use ambient_physics::physx::PhysicsKey;
use anyhow::Context;
use cli::Cli;
use log::LevelFilter;
use server::QUIC_INTERFACE_PORT;

fn main() -> anyhow::Result<()> {
    // Initialize the logger and lower the log level for modules we don't need to hear from by default.
    {
        const MODULES: &[(LevelFilter, &[&str])] = &[
            (
                LevelFilter::Error,
                &[
                    // Warns about extra syntactic elements; we are not concerned with these.
                    "fbxcel",
                ],
            ),
            (
                LevelFilter::Warn,
                &[
                    "ambient_build",
                    "ambient_gpu",
                    "ambient_model",
                    "ambient_physics",
                    "ambient_std",
                    "naga",
                    "tracing",
                    "wgpu_core",
                    "wgpu_hal",
                ],
            ),
        ];

        let mut builder = env_logger::builder();
        builder.filter_level(LevelFilter::Info);

        for (level, modules) in MODULES {
            for module in *modules {
                builder.filter_module(module, *level);
            }
        }

        builder.parse_default_env().try_init()?;
    }
    shared::components::init()?;
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    let assets = AssetCache::new(runtime.handle().clone());
    PhysicsKey.get(&assets); // Load physics
    AssetsCacheOnDisk.insert(&assets, false); // Disable disk caching for now; see https://github.com/AmbientRun/Ambient/issues/81

    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let project_path = cli.project().and_then(|p| p.path.clone()).unwrap_or_else(|| current_dir.clone());
    let project_path =
        if project_path.is_absolute() { project_path } else { ambient_std::path::normalize(&current_dir.join(project_path)) };

    if project_path.exists() && !project_path.is_dir() {
        anyhow::bail!("Project path {project_path:?} exists and is not a directory.");
    }

    // If new: create project, immediately exit
    if let Cli::New { name, .. } = &cli {
        if let Err(err) = cli::new_project::new_project(&project_path, name.as_deref()) {
            eprintln!("Failed to create project: {err:?}");
        }
        return Ok(());
    }

    // If UIC: write components to disk, immediately exit
    #[cfg(not(feature = "production"))]
    if let Cli::UpdateInterfaceComponents = cli {
        let toml = shared::components::dev::build_components_toml().to_string();

        // Assume we are being run within the codebase.
        for guest_path in std::fs::read_dir("guest/").unwrap().filter_map(Result::ok).map(|de| de.path()).filter(|de| de.is_dir()) {
            let toml_path = if guest_path.file_name().unwrap_or_default() == "rust" {
                guest_path.join("api").join("api_macros").join("ambient.toml")
            } else {
                guest_path.join("api").join("ambient.toml")
            };
            std::fs::write(&toml_path, &toml)?;
            log::info!("Interface updated at {toml_path:?}");
        }
        return Ok(());
    }

    // If a project was specified, assume that assets need to be built
    let manifest = cli
        .project()
        .map(|_| {
            anyhow::Ok(ambient_project::Manifest::parse(
                &std::fs::read_to_string(project_path.join("ambient.toml")).context("No project manifest was found. Please create one.")?,
            )?)
        })
        .transpose()?;

    if let Some(manifest) = manifest.as_ref() {
        let project_name = manifest.project.name.as_deref().unwrap_or("project");
        log::info!("Building {}", project_name);
        runtime.block_on(ambient_build::build(PhysicsKey.get(&assets), &assets, project_path.clone(), manifest));
        log::info!("Done building {}", project_name);
    }

    // If this is just a build, exit now
    if matches!(&cli, Cli::Build { .. }) {
        return Ok(());
    }

    // Otherwise, either connect to a server or host one
    let server_addr = if let Cli::Join { host, .. } = &cli {
        if let Some(mut host) = host.clone() {
            if !host.contains(':') {
                host = format!("{host}:{QUIC_INTERFACE_PORT}");
            }
            host.parse().with_context(|| format!("Invalid address for host {host}"))?
        } else {
            format!("127.0.0.1:{QUIC_INTERFACE_PORT}").parse()?
        }
    } else {
        let port = server::start(&runtime, assets.clone(), cli.clone(), project_path, manifest.as_ref().expect("no manifest"));
        format!("127.0.0.1:{port}").parse()?
    };

    // Time to join!
    let handle = runtime.handle().clone();
    if let Some(run) = cli.run() {
        // If we have run parameters, start a client and join a server
        runtime.block_on(client::run(assets, server_addr, run));
    } else {
        // Otherwise, wait for the Ctrl+C signal
        handle.block_on(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {}
                Err(err) => log::error!("Unable to listen for shutdown signal: {}", err),
            }
        });
    }
    Ok(())
}
