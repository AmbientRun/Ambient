use campfire::{
    cli::Cli,
    doc, example, golden_images, install, release,
    web::{self, Web},
};
use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
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
