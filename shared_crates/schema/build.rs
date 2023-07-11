use std::path::PathBuf;

fn main() {
    let files = std::iter::once(PathBuf::from("src/ambient.toml"))
        .chain(
            std::fs::read_dir("src/schema")
                .unwrap()
                .filter_map(Result::ok)
                .map(|de| de.path()),
        )
        .filter(|path| path.extension().map(|ext| ext == "toml").unwrap_or(false))
        .map(|path| {
            (
                path.strip_prefix("src")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                std::fs::read_to_string(&path).unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("schema.rs");
    std::fs::write(
        dest_path,
        format!(
            "pub const FILES: &[(&str, &str)] = &{:?};",
            files
                .iter()
                .map(|(path, contents)| (path.as_str(), contents.as_str()))
                .collect::<Vec<_>>()
        ),
    )
    .unwrap();
}
