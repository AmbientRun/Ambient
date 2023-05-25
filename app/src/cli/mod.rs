use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

pub mod new_project;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Clone, Debug)]
pub enum Commands {
    /// Create a new Ambient project
    New {
        #[command(flatten)]
        project_args: ProjectCli,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Builds and runs the project locally
    Run {
        #[command(flatten)]
        project_args: ProjectCli,
        #[command(flatten)]
        host_args: HostCli,
        #[command(flatten)]
        run_args: RunCli,
    },
    /// Builds the project
    Build {
        #[command(flatten)]
        project_args: ProjectCli,
    },
    /// Deploys the project
    #[cfg(feature = "deploy")]
    Deploy {
        #[command(flatten)]
        project_args: ProjectCli,
        /// API server endpoint, defaults to https://api.ambient.run
        #[arg(long)]
        api_server: Option<String>,
        /// Authentication token
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Builds and runs the project in server-only mode
    Serve {
        #[command(flatten)]
        project_args: ProjectCli,
        #[command(flatten)]
        host_args: HostCli,
    },
    /// View an asset
    View {
        #[command(flatten)]
        project_args: ProjectCli,
        /// Relative to the project path
        asset_path: PathBuf,
    },
    /// Join a multiplayer session
    Join {
        #[command(flatten)]
        run_args: RunCli,
        /// The server to connect to; defaults to localhost
        host: Option<String>,
    },
}

#[derive(Subcommand, Clone, Copy, Debug)]
pub enum GoldenImageCommand {
    /// Renders an image and saves it after waiting for the specified number of seconds
    GoldenImageUpdate {
        #[arg(long)]
        wait_seconds: f32,
    },
    /// Renders an image and compares it against existing golden image and timeouts after the specified number seconds
    GoldenImageCheck {
        #[arg(long)]
        timeout_seconds: f32,
    },
}

#[derive(Args, Clone, Debug)]
pub struct RunCli {
    /// If set, show a debugger that can be used to investigate the state of the project. Can also be accessed through the `AMBIENT_DEBUGGER` environment variable
    #[arg(short, long)]
    pub debugger: bool,

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
pub struct ProjectCli {
    /// The path or URL of the project to run; if not specified, this will default to the current directory
    pub path: Option<String>,

    /// Build all the assets with full optimization; this will make debugging more difficult
    #[arg(short, long)]
    pub release: bool,

    /// Avoid building the project
    #[arg(long)]
    pub no_build: bool,
}
#[derive(Args, Clone, Debug)]
pub struct HostCli {
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
            #[cfg(feature = "deploy")]
            Commands::Deploy { .. } => None,
            Commands::Serve { .. } => None,
            Commands::View { .. } => None,
            Commands::Join { run_args, .. } => Some(run_args),
        }
    }
    /// Extract project-relevant state only
    pub fn project(&self) -> Option<&ProjectCli> {
        match &self.command {
            Commands::New { project_args, .. } => Some(project_args),
            Commands::Run { project_args, .. } => Some(project_args),
            Commands::Build { project_args, .. } => Some(project_args),
            #[cfg(feature = "deploy")]
            Commands::Deploy { project_args, .. } => Some(project_args),
            Commands::Serve { project_args, .. } => Some(project_args),
            Commands::View { project_args, .. } => Some(project_args),
            Commands::Join { .. } => None,
        }
    }
    /// Extract host-relevant state only
    pub fn host(&self) -> Option<&HostCli> {
        match &self.command {
            Commands::New { .. } => None,
            Commands::Run { host_args, .. } => Some(host_args),
            Commands::Build { .. } => None,
            #[cfg(feature = "deploy")]
            Commands::Deploy { .. } => None,
            Commands::Serve { host_args, .. } => Some(host_args),
            Commands::View { .. } => None,
            Commands::Join { .. } => None,
        }
    }
}
