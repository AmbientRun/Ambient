use std::process::ExitCode;

pub mod cli;
pub mod doc;
pub mod golden_images;
pub mod install;
pub mod join;
pub mod package;
pub mod release;
pub mod util;
pub mod web;

use clap::Parser;
use cli::Cli;
use tracing_subscriber::filter::LevelFilter;

async fn run() -> anyhow::Result<()> {
    // A very minimal and short log output
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(LevelFilter::INFO)
        .without_time()
        .with_target(false)
        .init();

    if !std::path::Path::new("schema/schema/ambient.toml").exists() {
        anyhow::bail!("ambient.toml not found. Please run this from the root of the Ambient repository (preferably using `cargo campfire`).");
    }

    let cli = Cli::parse();

    match cli {
        Cli::Doc(doc) => doc::main(&doc),
        Cli::Package(p) => package::main(&p),
        Cli::GoldenImages(gi) => golden_images::main(&gi).await,
        Cli::Release(re) => release::main(&re),
        Cli::Install(install) => install::main(&install),
        Cli::Join(join) => join::main(&join),

        Cli::Clean => package::clean(),
        Cli::Run(run) => package::run(&run),
        Cli::Serve(run) => package::serve(&run),
        Cli::Web(command) => web::run(command).await,
    }
}

fn main() -> ExitCode {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    match rt.block_on(run()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}
