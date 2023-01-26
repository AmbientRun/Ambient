use std::path::{Path, PathBuf};

pub fn bundle_scripting_interface(
    // should be relative
    interface_path: &Path,
    // should include extension: "interface.json"
    output_file_name: &str,
    // additional directories to include
    directories: &[&str],
) {
    // Store TARGET for rustup fetch
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        interface_path.as_os_str().to_string_lossy()
    );

    let walk_dir = |path| {
        walkdir::WalkDir::new(interface_path.join(path))
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
    .map(|p| interface_path.join(p))
    .chain(directories.iter().flat_map(walk_dir))
    .filter(|p| p.is_file())
    .map(|p| {
        (
            p.strip_prefix(interface_path).unwrap().to_owned(),
            std::fs::read_to_string(p).unwrap(),
        )
    })
    .collect();

    std::fs::write(
        Path::new(&std::env::var("OUT_DIR").unwrap()).join(output_file_name),
        serde_json::to_string(&files).unwrap(),
    )
    .unwrap();
}
