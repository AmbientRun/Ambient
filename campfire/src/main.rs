use clap::Parser;

mod doc;
mod example;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cli {
    /// Generate documentation for Ambient
    Doc,
    /// Example-related functionality
    #[command(subcommand)]
    Example(example::Example),
}

fn main() -> anyhow::Result<()> {
    if !std::path::Path::new("ambient.toml").exists() {
        anyhow::bail!("ambient.toml not found. Please run this from the root of the Ambient repository (preferably using `cargo run -p campfire`).");
    }

    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, Default::default())?;

    let cli = Cli::parse();

    match cli {
        Cli::Doc => {
            doc::main()?;
        }
        Cli::Example(ex) => example::main(&ex)?,
    }

    Ok(())
}
