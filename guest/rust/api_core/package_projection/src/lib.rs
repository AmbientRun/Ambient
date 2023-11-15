use ambient_package_macro_common as apmc;
use std::path::Path;

pub fn generate() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("no manifest dir");
    let manifest_dir = Path::new(&manifest_dir);

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("ambient.toml").display()
    );

    let generated = generate_impl(manifest_dir)
        .unwrap_or_else(|e| {
            let msg = format!(
                "Error while running Ambient package macro: {e}{}",
                e.source()
                    .map(|e| format!("\nCaused by: {e}"))
                    .unwrap_or_default()
            );
            quote::quote! {
                compile_error!(#msg);
            }
        })
        .to_string();

    // write generation to manifest_dir/src/packages.rs
    let packages_rs = manifest_dir.join("src").join("packages.rs");
    std::fs::write(&packages_rs, generated).expect("failed to write packages.rs");

    std::process::Command::new("rustfmt")
        .arg(&packages_rs)
        .status()
        .expect("failed to run rustfmt on packages.rs");
}

fn generate_impl(manifest_dir: &Path) -> anyhow::Result<proc_macro2::TokenStream> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(anyhow::Error::new)
        .and_then(|rt| {
            rt.block_on(apmc::generate_code(
                Some(apmc::RetrievableFile::Path(
                    manifest_dir.join("ambient.toml"),
                )),
                apmc::Context::GuestUser,
            ))
        })
}
