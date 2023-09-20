use std::{net::IpAddr, path::PathBuf};

use clap::Args;

use super::PackagePath;

pub mod build;
pub mod deploy;
pub mod new;
pub mod run;
pub mod serve;

#[derive(Args, Clone, Debug)]
pub struct PackageCli {
    /// Dummy flag to catch Rust users using muscle memory and warn them
    #[arg(long, short, hide = true)]
    pub project: bool,

    /// The path or URL of the package to run; if not specified, this will default to the current directory
    pub path: Option<String>,

    /// Build all the assets with debug information; this will make them less performant and larger but easier to debug (default for all commands apart from `deploy` and `serve`)
    #[arg(long, conflicts_with = "release")]
    debug: bool,

    /// Build all the assets with full optimization; this will make them faster and smaller but more difficult to debug (default for `deploy` and `serve`)
    #[arg(short, long)]
    release: bool,

    /// Avoid building the package
    #[arg(long)]
    pub no_build: bool,

    #[arg(long)]
    /// Perform a clean build
    pub clean_build: bool,

    #[arg(long)]
    /// Only build the WASM modules
    pub build_wasm_only: bool,
}
impl PackageCli {
    pub fn is_release(&self) -> Option<bool> {
        match (self.debug, self.release) {
            (true, false) => Some(false),
            (false, true) => Some(true),
            (false, false) => None,
            (true, true) => {
                // clap's conflict_with should prevent this from happening
                panic!("debug and release are mutually exclusive")
            }
        }
    }

    pub fn package_path(&self) -> anyhow::Result<PackagePath> {
        self.path.clone().try_into()
    }
}

#[derive(Args, Clone, Debug)]
pub struct HostCli {
    #[arg(long, default_value = "0.0.0.0")]
    pub bind_address: IpAddr,
    /// Provide a public address or IP to the instance, which will allow users to connect to this instance over the internet
    ///
    /// Defaults to localhost
    #[arg(long)]
    pub public_host: Option<String>,

    /// Defaults to 8889
    #[arg(long)]
    pub http_interface_port: Option<u16>,

    /// Defaults to 9000
    #[arg(long)]
    pub quic_interface_port: Option<u16>,

    /// Don't use proxy for NAT traversal
    #[arg(long)]
    pub no_proxy: bool,

    /// AmbientProxy address to use for NAT traversal
    #[arg(long)]
    pub proxy: Option<String>,

    /// Pre-cache assets on the proxy
    #[arg(long)]
    pub proxy_pre_cache_assets: bool,

    /// Certificate for TLS
    #[arg(long, requires("key"))]
    pub cert: Option<PathBuf>,
    /// Private key for the certificate
    #[arg(long)]
    pub key: Option<PathBuf>,

    /// Shutdown the server after the specified number of seconds of inactivity
    #[arg(long)]
    pub shutdown_after_inactivity_seconds: Option<u64>,
}
