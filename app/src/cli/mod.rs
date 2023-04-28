use std::path::PathBuf;

use clap::{Args, Parser};

pub mod new_project;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cli {
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
#[derive(Args, Clone)]
pub struct RunCli {
    /// If set, show a debugger that can be used to investigate the state of the project. Can also be accessed through the `AMBIENT_DEBUGGER` environment variable
    #[arg(short, long)]
    pub debugger: bool,

    /// Run in headless mode
    #[arg(long)]
    pub headless: bool,

    /// Take a screenshot after N seconds, compare it to the existing one and then exit with an exit code of 1 if they are different
    #[arg(long)]
    pub golden_image_test: Option<f32>,

    /// The user ID to join this server with
    #[clap(short, long)]
    pub user_id: Option<String>,
}
#[derive(Args, Clone)]
pub struct ProjectCli {
    /// The path of the project to run; if not specified, this will default to the current directory
    pub path: Option<PathBuf>,

    /// Build all the assets with full optimization; this will make debugging more difficult
    #[arg(short, long)]
    pub release: bool,

    /// Avoid building the project
    #[arg(long)]
    pub no_build: bool,
}
#[derive(Args, Clone)]
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
}

impl Cli {
    /// Extract run-relevant state only
    pub fn run(&self) -> Option<&RunCli> {
        match self {
            Cli::New { .. } => None,
            Cli::Run { run_args, .. } => Some(run_args),
            Cli::Build { .. } => None,
            #[cfg(feature = "deploy")]
            Cli::Deploy { .. } => None,
            Cli::Serve { .. } => None,
            Cli::View { .. } => None,
            Cli::Join { run_args, .. } => Some(run_args),
        }
    }
    /// Extract project-relevant state only
    pub fn project(&self) -> Option<&ProjectCli> {
        match self {
            Cli::New { project_args, .. } => Some(project_args),
            Cli::Run { project_args, .. } => Some(project_args),
            Cli::Build { project_args, .. } => Some(project_args),
            #[cfg(feature = "deploy")]
            Cli::Deploy { project_args, .. } => Some(project_args),
            Cli::Serve { project_args, .. } => Some(project_args),
            Cli::View { project_args, .. } => Some(project_args),
            Cli::Join { .. } => None,
        }
    }
    /// Extract host-relevant state only
    pub fn host(&self) -> Option<&HostCli> {
        match self {
            Cli::New { .. } => None,
            Cli::Run { host_args, .. } => Some(host_args),
            Cli::Build { .. } => None,
            #[cfg(feature = "deploy")]
            Cli::Deploy { .. } => None,
            Cli::Serve { host_args, .. } => Some(host_args),
            Cli::View { .. } => None,
            Cli::Join { .. } => None,
        }
    }
}
