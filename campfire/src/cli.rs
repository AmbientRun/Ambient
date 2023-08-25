use clap::Parser;

use crate::{doc, example, golden_images, install, package, release, web};

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None, propagate_version = true, trailing_var_arg = true)]
pub enum Cli {
    /// Generate documentation for Ambient
    #[command(subcommand)]
    Doc(doc::Doc),
    /// Example-related functionality
    #[command(subcommand)]
    Example(example::Example),
    /// Package-related functionality
    #[command(subcommand)]
    Package(package::Package),
    /// Running golden image tests
    GoldenImages(golden_images::GoldenImages),
    /// Release-related functionality
    #[command(subcommand)]
    Release(release::Release),
    /// Helper to install specific versions of Ambient
    Install(install::Install),

    #[command(subcommand)]
    /// Web-related functionality
    Web(web::Web),

    // Helper aliases for subcommands
    /// Clean all build artifacts for all examples.
    Clean,
    /// Run an example. Alias for `example run`.
    Run(example::Run),
}

// https://rust-cli-recommendations.sunshowers.io/handling-arguments.html
