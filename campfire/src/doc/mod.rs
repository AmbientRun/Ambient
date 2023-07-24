use std::path::Path;

use anyhow::Context;
use clap::Parser;

mod context;
mod helpers;
mod parser;
mod typescript;

#[derive(Parser, Clone)]
#[clap(trailing_var_arg = true)]
pub enum Doc {
    /// Document the runtime
    Runtime,
    /// Document the API
    Api {
        #[clap(long, short)]
        /// Open the docs in a browser
        open: bool,

        /// The args to pass through to `cargo doc`
        args: Vec<String>,
    },
}

pub(crate) fn main(args: &Doc) -> anyhow::Result<()> {
    match args {
        Doc::Runtime => runtime(),
        Doc::Api { open, args } => api(*open, args),
    }
}

fn runtime() -> anyhow::Result<()> {
    pipeline()
}

fn api(open: bool, args: &[String]) -> anyhow::Result<()> {
    let root_path = Path::new("guest/rust");

    let mut command = std::process::Command::new("cargo");
    command.current_dir(root_path);
    command.args(["+nightly", "doc", "-p", "ambient_api", "--all-features"]);
    if open {
        command.arg("--open");
    }
    command.args(args.iter().map(|s| s.as_str()));
    command.env("RUSTDOCFLAGS", "--cfg docsrs");

    if !command.spawn()?.wait()?.success() {
        anyhow::bail!("Failed to document Rust API.");
    }

    Ok(())
}

fn pipeline() -> anyhow::Result<()> {
    log::info!("Generating pipeline.d.ts...");

    let ctx = context::Context::new(&[
        Path::new("crates/physics/Cargo.toml"),
        Path::new("crates/model_import/Cargo.toml"),
        Path::new("crates/build/Cargo.toml"),
    ])?;

    log::info!("Built context from rustdoc.");

    let (build_crate, pipeline) = ctx
        .get("ambient_build::pipelines::Pipeline")
        .context("no pipeline struct found")?;

    let ty = parser::Type::convert_item(&ctx, build_crate, pipeline);

    typescript::generate(
        &ty,
        std::path::Path::new("docs/src/reference/pipeline.d.ts"),
    )?;

    log::info!("Done generating pipeline.d.ts.");

    Ok(())
}
