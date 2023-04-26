use clap::Parser;

mod doc;
mod example;
mod release;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None, propagate_version = true, trailing_var_arg = true)]
pub enum Cli {
    /// Generate documentation for Ambient
    #[command(subcommand)]
    Doc(doc::Doc),
    /// Example-related functionality
    #[command(subcommand)]
    Example(example::Example),
    /// Release-related functionality
    #[command(subcommand)]
    Release(release::Release),

    // Helper aliases for subcommands
    /// Clean all build artifacts for all examples.
    Clean,
    /// Run an example. Alias for `example run`.
    Run(example::Run),
}

fn main() -> anyhow::Result<()> {
    if !std::path::Path::new("ambient.toml").exists() {
        anyhow::bail!("ambient.toml not found. Please run this from the root of the Ambient repository (preferably using `cargo campfire`).");
    }

    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, Default::default())?;

    let cli = Cli::parse();

    match cli {
        Cli::Doc(doc) => doc::main(&doc),
        Cli::Example(ex) => example::main(&ex),
        Cli::Release(re) => release::main(&re),

        Cli::Clean => example::clean(),
        Cli::Run(run) => example::run(&run),
    }
}
