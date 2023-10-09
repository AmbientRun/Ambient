use clap::Subcommand;

use self::build::BuildOptions;

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
    OpenBrowser(browser::BrowserOptions),
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
        Web::OpenBrowser(args) => browser::open(args).await,
        #[cfg(feature = "serve")]
        Web::Serve(args) => args.run().await,
    }
}
