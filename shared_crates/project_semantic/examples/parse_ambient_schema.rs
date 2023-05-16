use std::path::Path;

use ambient_project_semantic::{FileProvider, Semantic};

pub fn main() {
    const SCHEMA_PATH: &str = "shared_crates/schema/src";

    struct DiskFileProvider;
    impl FileProvider for DiskFileProvider {
        fn get(&self, filename: &str) -> std::io::Result<String> {
            std::fs::read_to_string(Path::new(SCHEMA_PATH).join(filename))
        }
    }

    let mut semantic = Semantic::new();
    semantic
        .add_file("ambient.toml", &DiskFileProvider)
        .unwrap();

    dbg!(semantic);
}
