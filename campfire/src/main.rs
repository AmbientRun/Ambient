use std::process::ExitCode;

use campfire::{
    cli::Cli,
    doc, example, golden_images, install, release,
    web::{self},
};
use clap::Parser;

async fn run() -> anyhow::Result<()> {
    if !std::path::Path::new("shared_crates/schema/src/ambient.toml").exists() {
        anyhow::bail!("ambient.toml not found. Please run this from the root of the Ambient repository (preferably using `cargo campfire`).");
    }

    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, Default::default())?;

    let cli = Cli::parse();

    match cli {
        Cli::Doc(doc) => doc::main(&doc),
        Cli::Example(ex) => example::main(&ex),
        Cli::GoldenImages(gi) => golden_images::main(&gi).await,
        Cli::Release(re) => release::main(&re),
        Cli::Install(install) => install::main(&install),

        Cli::Clean => example::clean(),
        Cli::Run(run) => example::run(&run),
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
