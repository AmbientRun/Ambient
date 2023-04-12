//! This code is extremely ugly. Its sole purpose is to convert our serde types
//! into something that can be used in documentation.
use std::collections::HashMap;

use anyhow::Context;
use once_cell::sync::Lazy;
use rustdoc_types::{Crate, Id};

mod helpers;
use helpers::*;
mod parser;
mod typescript;

static CRATES: Lazy<HashMap<String, Crate>> = Lazy::new(|| {
    [
        "crates/physics/Cargo.toml",
        "crates/model_import/Cargo.toml",
        "crates/build/Cargo.toml",
    ]
    .iter()
    .map(|n| {
        let build = rustdoc_json::Builder::default()
            .toolchain("nightly")
            .document_private_items(true)
            .manifest_path(n)
            .silent(true)
            .build()
            .unwrap();

        let krate = serde_json::from_str(&std::fs::read_to_string(build).unwrap()).unwrap();
        (n.to_string(), krate)
    })
    .collect()
});
static PATH_TO_CRATE_AND_ID: Lazy<HashMap<String, (String, Id)>> = Lazy::new(|| {
    CRATES
        .iter()
        .flat_map(|(n, krate)| {
            krate
                .paths
                .iter()
                .filter(|p| p.1.crate_id == 0)
                .map(|p| (p.1.path.join("::"), (n.clone(), p.0.clone())))
        })
        .collect()
});

pub(crate) fn main() -> anyhow::Result<()> {
    let (crate_path, id) = PATH_TO_CRATE_AND_ID
        .get("ambient_build::pipelines::Pipeline")
        .context("no pipeline struct found")?;
    let build_crate = CRATES.get(crate_path).unwrap();
    let pipeline = id.get(build_crate);
    let ty = parser::Type::convert_item(build_crate, pipeline);

    typescript::generate(
        &ty,
        std::path::Path::new("docs/src/reference/pipeline.d.ts"),
    )
}
