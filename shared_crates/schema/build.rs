use std::path::PathBuf;

fn main() {
    const SCHEMA_LOCATION: &str = "../../schema";
    println!("cargo:rerun-if-changed={SCHEMA_LOCATION}");

    let files = std::iter::once(PathBuf::from(format!("{SCHEMA_LOCATION}/ambient.toml")))
        .chain(
            std::fs::read_dir(format!("{SCHEMA_LOCATION}/schema"))
                .unwrap()
                .filter_map(Result::ok)
                .map(|de| de.path()),
        )
        .filter(|path| path.extension().map(|ext| ext == "toml").unwrap_or(false))
        .map(|path| {
            (
                path.strip_prefix(SCHEMA_LOCATION)
                    .unwrap()
                    .iter()
                    .map(|s| s.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/"),
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
