use std::process::ExitCode;

use campfire::{cli::Cli, doc, golden_images, install, package, release, web};
use clap::Parser;

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

        Cli::Clean => package::clean(),
        Cli::Run(run) => package::run(&run),
        Cli::Web(command) => web::run(command).await,
    }
}

fn main() -> ExitCode {
    let rt = tokio::runtime::Builder::new_multi_thread()
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
