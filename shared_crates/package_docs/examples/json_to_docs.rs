use std::path::PathBuf;

// Suggested to run with:
// `cargo watch -x 'run -p ambient_package_docs --example json_to_docs -- path/to/ambient_package.json' -w shared_crates/package_docs/src`
fn main() -> anyhow::Result<()> {
    let json_path = PathBuf::from(std::env::args().nth(1).expect("path as first arg"));
    assert_eq!(json_path.extension().expect("extension"), "json");
    let build_path = json_path.parent().expect("parent dir");

    let docs_path = build_path.join("docs");
    std::fs::remove_dir_all(&docs_path).ok();
    std::fs::create_dir_all(&docs_path)?;
    ambient_package_docs::write(&docs_path, &json_path, true)?;

    Ok(())
}
