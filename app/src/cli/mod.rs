use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

pub mod assets;
pub mod join;
pub mod login;
pub mod package;

mod package_path;
pub use package_path::*;

use self::{
    assets::Assets,
    join::Join,
    package::{build::Build, deploy::Deploy, new::New, run::Run, serve::Serve, PackageCli},
};

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Clone, Debug)]
pub enum Commands {
    New(New),
    Run(Run),
    Build(Build),
    Deploy(Deploy),
    Serve(Serve),
    Join(Join),
    /// Asset manipulation and migration
    Assets {
        #[command(subcommand)]
        assets: Assets,
    },
    /// Log into Ambient and save your API token to settings
    Login,
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
pub struct ClientCli {
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

    /// Window position X override
    #[arg(long)]
    pub window_x: Option<i32>,

    /// Window position Y override
    #[arg(long)]
    pub window_y: Option<i32>,

    /// Window width override
    #[arg(long)]
    pub window_width: Option<u32>,

    /// Window height override
    #[arg(long)]
    pub window_height: Option<u32>,
}

impl Cli {
    /// Extract package-relevant state only
    pub fn package(&self) -> Option<&PackageCli> {
        match &self.command {
            Commands::New(New { package, .. }) => Some(package),
            Commands::Run(Run { package, .. }) => Some(package),
            Commands::Build(Build { package, .. }) => Some(package),
            Commands::Deploy(Deploy { package, .. }) => Some(package),
            Commands::Serve(Serve { package, .. }) => Some(package),
            Commands::Join(Join { .. }) => None,
            Commands::Assets { .. } => None,
            Commands::Login => None,
        }
    }
    pub fn use_release_build(&self) -> bool {
        use Commands as C;

        match &self.command {
            C::Deploy(Deploy { package, .. }) | C::Serve(Serve { package, .. }) => {
                package.is_release().unwrap_or(true)
            }
            C::Run(Run { package, .. }) | C::Build(Build { package, .. }) => {
                package.is_release().unwrap_or(false)
            }
            C::New(_) | C::Join(_) | C::Assets { .. } | C::Login => false,
        }
    }
}
