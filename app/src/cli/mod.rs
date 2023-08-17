use std::{net::IpAddr, path::PathBuf};

use clap::{Args, Parser, Subcommand};

pub mod new_package;

pub mod assets;
pub mod build;
pub mod client;
pub mod deploy;
pub mod server;

mod package_path;
pub use package_path::*;

use self::assets::AssetCommand;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Clone, Debug)]
pub enum Commands {
    /// Create a new Ambient package
    New {
        #[command(flatten)]
        package_args: PackageCli,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(long)]
        api_path: Option<String>,
    },
    /// Builds and runs the package locally
    Run {
        #[command(flatten)]
        package_args: PackageCli,
        #[command(flatten)]
        host_args: HostCli,
        #[command(flatten)]
        run_args: RunCli,
    },
    /// Builds the package
    Build {
        #[command(flatten)]
        package_args: PackageCli,
    },
    /// Deploys the package
    Deploy {
        #[command(flatten)]
        package_args: PackageCli,
        /// API server endpoint
        #[arg(long, default_value = "https://api.ambient.run")]
        api_server: String,
        /// Authentication token
        #[arg(short, long, required = true)]
        token: String,
        /// Don't use differential upload and upload all assets
        #[arg(long)]
        force_upload: bool,
        /// Ensure the package is running after deploying
        #[arg(long)]
        ensure_running: bool,
        /// Context to run the package in
        #[arg(long, requires("ensure_running"), default_value = "")]
        context: String,
    },
    /// Builds and runs the package in server-only mode
    Serve {
        #[command(flatten)]
        package_args: PackageCli,
        #[command(flatten)]
        host_args: HostCli,
    },
    /// View an asset
    View {
        #[command(flatten)]
        package_args: PackageCli,
        /// Relative to the package path
        asset_path: PathBuf,
    },
    /// Join a multiplayer session
    Join {
        #[command(flatten)]
        run_args: RunCli,
        /// The server to connect to; defaults to localhost
        host: Option<String>,
    },
    /// Asset manipulation and migration
    Assets {
        #[command(subcommand)]
        command: AssetCommand,
    },
}

#[derive(Subcommand, Clone, Copy, Debug)]
pub enum GoldenImageCommand {
    /// Renders an image and saves it after waiting for the specified number of seconds
    #[command(name = "golden-image-update")]
    Update {
        #[arg(long)]
        wait_seconds: f32,
    },
    /// Renders an image and compares it against existing golden image and timeouts after the specified number seconds
    #[command(name = "golden-image-check")]
    Check {
        #[arg(long)]
        timeout_seconds: f32,
    },
}

#[derive(Args, Clone, Debug)]
pub struct RunCli {
    /// If set, show a debugger that can be used to investigate the state of the package.
    /// Can also be accessed through the `AMBIENT_DEBUGGER` environment variable
    #[arg(short, long)]
    pub debugger: bool,

    /// If set, no audio will be played, which can be useful for debugging
    #[arg(long)]
    pub mute_audio: bool,

    /// Run in headless mode
    #[arg(long)]
    pub headless: bool,

    /// Run golden image test
    #[command(subcommand)]
    pub golden_image: Option<GoldenImageCommand>,

    /// The user ID to join this server with
    #[clap(short, long)]
    pub user_id: Option<String>,

    /// Specify a trusted certificate authority
    #[arg(long)]
    pub ca: Option<PathBuf>,
}

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
}

impl Cli {
    /// Extract run-relevant state only
    pub fn run(&self) -> Option<&RunCli> {
        match &self.command {
            Commands::New { .. } => None,
            Commands::Run { run_args, .. } => Some(run_args),
            Commands::Build { .. } => None,
            Commands::Deploy { .. } => None,
            Commands::Serve { .. } => None,
            Commands::View { .. } => None,
            Commands::Join { run_args, .. } => Some(run_args),
            Commands::Assets { .. } => None,
        }
    }
    /// Extract package-relevant state only
    pub fn package(&self) -> Option<&PackageCli> {
        match &self.command {
            Commands::New { package_args, .. } => Some(package_args),
            Commands::Run { package_args, .. } => Some(package_args),
            Commands::Build { package_args, .. } => Some(package_args),
            Commands::Deploy { package_args, .. } => Some(package_args),
            Commands::Serve { package_args, .. } => Some(package_args),
            Commands::View { package_args, .. } => Some(package_args),
            Commands::Join { .. } => None,
            Commands::Assets { .. } => None,
        }
    }
    /// Extract host-relevant state only
    pub fn host(&self) -> Option<&HostCli> {
        match &self.command {
            Commands::New { .. } => None,
            Commands::Run { host_args, .. } => Some(host_args),
            Commands::Build { .. } => None,
            Commands::Deploy { .. } => None,
            Commands::Serve { host_args, .. } => Some(host_args),
            Commands::View { .. } => None,
            Commands::Join { .. } => None,
            Commands::Assets { .. } => None,
        }
    }
    pub fn use_release_build(&self) -> bool {
        match &self.command {
            Commands::Deploy { package_args, .. } | Commands::Serve { package_args, .. } => {
                package_args.is_release().unwrap_or(true)
            }
            Commands::Run { package_args, .. }
            | Commands::Build { package_args, .. }
            | Commands::View { package_args, .. } => package_args.is_release().unwrap_or(false),
            Commands::New { .. } | Commands::Join { .. } | Commands::Assets { .. } => false,
        }
    }
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
}
