use std::path::PathBuf;

use clap::{Parser, Subcommand};
use elements_app::{App, AppBuilder};
use elements_ecs::World;
use elements_model_import::{model_crate::ModelCrate, MODEL_EXTENSIONS};
use walkdir::WalkDir;

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

async fn build(path: PathBuf) {
    let target = path.join("target");
    let assets = path.join("assets");
    for model_path in WalkDir::new(assets)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|x| x.into_path())
        .filter(|e| MODEL_EXTENSIONS.iter().find(|x| *x == &e.extension().unwrap().to_str().unwrap().to_lowercase()).is_some())
    {
        // let model = ModelCrate::new();
    }
}

fn init(world: &mut World) {}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    let cli = Cli::parse();
    if let Some(path) = cli.command.get_build() {
        runtime.block_on(build(path));
    }

    AppBuilder::simple().with_runtime(runtime).run(|app, runtime| {
        runtime.spawn(async move {});
    });
}
