use clap::Subcommand;

use self::build::BuildOptions;

#[cfg(feature = "openssl")]
mod browser;
mod build;
#[cfg(feature = "serve")]
mod serve;

#[derive(Debug, Subcommand, Clone)]
pub enum Web {
    /// Build the web client to WebAssembly
    Build(BuildOptions),
    Check(BuildOptions),
    /// Launches chrome with the correct flags to explicitly trust
    /// the self-signed certificate
    OpenBrowser,
    #[cfg(feature = "serve")]
    Serve(serve::Serve),
}

pub async fn run(command: Web) -> anyhow::Result<()> {
    match command {
        Web::Build(args) => {
            args.build().await?;
            Ok(())
        }
        Web::Check(args) => args.check().await,
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
        #[cfg(feature = "serve")]
        Web::Serve(args) => args.run().await,
    }
}
