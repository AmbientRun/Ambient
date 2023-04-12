use std::path::Path;

use anyhow::Context;

mod context;
mod helpers;
mod parser;
mod typescript;

pub(crate) fn main() -> anyhow::Result<()> {
    pipeline()
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
