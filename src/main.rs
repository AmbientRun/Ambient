use std::{path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};
use elements_app::{App, AppBuilder};
use elements_ecs::{SimpleComponentRegistry, World};
use elements_std::asset_cache::AssetCache;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Builds and runs the project at the designated path
    Run { path: Option<String> },
    /// Builds the project at the designated path
    Build { path: Option<String> },
}
impl Commands {
    fn get_build(&self) -> Option<PathBuf> {
        let cur_dir = std::env::current_dir().unwrap();
        match self.clone() {
            Commands::Run { path } => Some(path.map(|x| x.into()).unwrap_or(cur_dir)),
            Commands::Build { path } => Some(path.map(|x| x.into()).unwrap_or(cur_dir)),
        }
    }
}

fn init(world: &mut World) {}

fn main() {
    SimpleComponentRegistry::install();
    elements_app::init_all_components();
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let assets = AssetCache::new(runtime.handle().clone());

    let cli = Cli::parse();
    if let Some(path) = cli.command.get_build() {
        runtime.block_on(elements_build::build(&assets, path));
    }

    if let Commands::Run { path } = cli.command {
        AppBuilder::simple().install_component_registry(false).with_runtime(runtime).with_asset_cache(assets).run(|app, runtime| {
            runtime.spawn(async move {});
        });
    }
}
