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

async fn run() -> anyhow::Result<()> {
    if !std::path::Path::new("schema/ambient.toml").exists() {
        anyhow::bail!("ambient.toml not found. Please run this from the root of the Ambient repository (preferably using `cargo campfire`).");
    }

    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, Default::default())?;

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
            log::error!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}
