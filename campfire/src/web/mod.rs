use clap::Subcommand;

use self::build::BuildOptions;

pub mod build;

#[derive(Debug, Subcommand, Clone)]
pub enum Web {
    /// Build the web client to WebAssembly
    Build {
        #[command(flatten)]
        args: BuildOptions,
    },
}

pub async fn run(command: Web) -> anyhow::Result<()> {
    match command {
        Web::Build { args } => build::run(args).await,
    }
}
