use clap::Parser;

mod doc;
mod example;
mod release;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cli {
    /// Generate documentation for Ambient
    Doc,
    /// Example-related functionality
    #[command(subcommand)]
    Example(example::Example),
    /// Release-related functionality
    #[command(subcommand)]
    Release(release::Release),
}

fn main() -> anyhow::Result<()> {
    if !std::path::Path::new("ambient.toml").exists() {
        anyhow::bail!("ambient.toml not found. Please run this from the root of the Ambient repository (preferably using `cargo campfire`).");
    }

    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, Default::default())?;

    let cli = Cli::parse();

    match cli {
        Cli::Doc => doc::main(),
        Cli::Example(ex) => example::main(&ex),
        Cli::Release(re) => release::main(&re),
    }
}
