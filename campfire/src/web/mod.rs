use clap::Subcommand;

use self::build::BuildOptions;

#[cfg(feature = "openssl")]
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
        Web::Build(args) => build::run(args),
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
    }
}
