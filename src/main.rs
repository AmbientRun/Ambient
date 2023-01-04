use std::{path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};
use elements_app::{App, AppBuilder};
use elements_ecs::{SimpleComponentRegistry, World};
use elements_model_import::{model_crate::ModelCrate, MODEL_EXTENSIONS};
use elements_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use futures::FutureExt;
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

async fn build(assets: &AssetCache, path: PathBuf) {
    let target = path.join("target");
    let assets_path = path.join("assets");
    for model_path in WalkDir::new(&assets_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|x| x.into_path())
        .filter(|e| MODEL_EXTENSIONS.iter().any(|x| x == &e.extension().unwrap_or_default().to_str().unwrap().to_lowercase()))
    {
        println!("model: {:?} {:?}", model_path, model_path.extension());
        let mut model = ModelCrate::new();
        model
            .import(assets, &AbsAssetUrl::from_file_path(&model_path), true, false, Arc::new(|path| async move { None }.boxed()))
            .await
            .unwrap();
        model.update_node_primitive_aabbs_from_cpu_meshes();
        model.model_mut().update_model_aabb();
        model.write_to_fs(&target.join("assets").join(&model_path.strip_prefix(&assets_path).unwrap())).await;
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
        runtime.block_on(build(&assets, path));
    }

    AppBuilder::simple().install_component_registry(false).with_runtime(runtime).with_asset_cache(assets).run(|app, runtime| {
        runtime.spawn(async move {});
    });
}
