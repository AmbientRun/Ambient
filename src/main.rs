use std::{path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};
use elements_app::{App, AppBuilder};
use elements_ecs::{SimpleComponentRegistry, World};
use elements_std::asset_cache::AssetCache;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Set the path to the project. Defaults to the current directory
    #[arg(short, long)]
    project_path: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Builds and runs the project
    Run,
    /// Builds the project
    Build,
    /// View an asset
    View { asset_path: String },
}
impl Commands {
    fn should_build(&self) -> bool {
        match self {
            Commands::Run => true,
            Commands::Build => true,
            Commands::View { .. } => true,
        }
    }
    fn should_run(&self) -> bool {
        match self {
            Commands::Run => true,
            Commands::Build => false,
            Commands::View { .. } => true,
        }
    }
}

fn main() {
    SimpleComponentRegistry::install();
    elements_app::init_all_components();
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let assets = AssetCache::new(runtime.handle().clone());

    let cli = Cli::parse();
    let project_path = cli.project_path.map(|x| x.into()).unwrap_or_else(|| std::env::current_dir().unwrap());
    if cli.command.should_build() {
        runtime.block_on(elements_build::build(&assets, project_path));
    }

    if cli.command.should_run() {
        AppBuilder::simple().install_component_registry(false).ui_renderer(true).with_runtime(runtime).with_asset_cache(assets).run(
            |app, runtime| {
                if let Commands::View { asset_path } = cli.command.clone() {
                    runtime.spawn(async move {});
                }
            },
        );
    }
}
