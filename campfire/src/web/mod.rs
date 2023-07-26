use clap::Subcommand;

use self::build::BuildOptions;

mod browser;
mod build;

#[derive(Debug, Subcommand, Clone)]
pub enum Web {
    /// Build the web client to WebAssembly
    Build(BuildOptions),
    /// Launches chrome with the correct flags to explicitly trust
    /// the self-signed certificate
    OpenBrowser,
}

pub async fn run(command: Web) -> anyhow::Result<()> {
    match command {
        Web::Build(args) => build::run(args).await,
        Web::OpenBrowser => browser::open().await,
    }
}
