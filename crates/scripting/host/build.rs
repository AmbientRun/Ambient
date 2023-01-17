use std::path::{Path, PathBuf};

fn main() {
    // Store TARGET for rustup fetch
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    let prefix = std::path::Path::new("../guest/rust");
    println!(
        "cargo:rerun-if-changed={}",
        prefix.as_os_str().to_string_lossy()
    );

    let walk_dir = |path| {
        walkdir::WalkDir::new(prefix.join(path))
            .into_iter()
            .filter_map(Result::ok)
            .map(|d| d.into_path())
    };

    // Store the Rust scripting interface for use on the client and server.
    let files: Vec<(PathBuf, String)> = [
        ".vscode/settings.json",
        "Cargo.lock",
        "Cargo.toml",
        "rust-toolchain.toml",
        "rustfmt.toml",
    ]
    .into_iter()
    .map(|p| prefix.join(p))
    .chain(walk_dir("src"))
    .chain(walk_dir("main-macro"))
    .filter(|p| p.is_file())
    .map(|p| {
        (
            p.strip_prefix(prefix).unwrap().to_owned(),
            std::fs::read_to_string(p).unwrap(),
        )
    })
    .collect();

    std::fs::write(
        Path::new(&std::env::var("OUT_DIR").unwrap()).join("elements_guest_rust_interface.json"),
        serde_json::to_string(&files).unwrap(),
    )
    .unwrap();
}
