use clap::Subcommand;

use self::{build::BuildOptions, serve::Serve};

#[cfg(feature = "openssl")]
mod browser;
mod build;
mod serve;

#[derive(Debug, Subcommand, Clone)]
pub enum Web {
    /// Build the web client to WebAssembly
    Build(BuildOptions),
    /// Launches chrome with the correct flags to explicitly trust
    /// the self-signed certificate
    OpenBrowser,
    Serve(Serve),
}

pub async fn run(command: Web) -> anyhow::Result<()> {
    match command {
        Web::Build(args) => build::run(&args).await,
        Web::OpenBrowser => {
            #[cfg(feature = "openssl")]
            {
                browser::open().await
            }
            #[cfg(not(feature = "openssl"))]
            {
                anyhow::bail!("The `openssl` feature must be enabled to use this command")
            }
        }
        Web::Serve(args) => args.run().await,
    }
}
