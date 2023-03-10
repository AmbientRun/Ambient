use std::path::{Path, PathBuf};

struct File {
    absolute_path: PathBuf,
    contents: String,
}

/// Copies all files in the wit/ folder to all guests, and generates wit ahead of time
/// for Rust guests.
fn main() {
    let working_dir = std::env::current_dir().unwrap();

    println!("cargo:rerun-if-changed=wit");
    let filenames_to_copy: Vec<_> = std::fs::read_dir("wit")
        .unwrap()
        .map(|r| r.map(|de| de.path()))
        .collect::<Result<_, _>>()
        .unwrap();

    let files: Vec<_> = filenames_to_copy
        .iter()
        .map(|path| -> std::io::Result<_> {
            let absolute_path = working_dir.join(path).canonicalize().unwrap();

            // De-UNC the path.
            #[cfg(target_os = "windows")]
            let absolute_path = dunce::simplified(&absolute_path).to_owned();

            Ok(File {
                absolute_path,
                contents: std::fs::read_to_string(path)?,
            })
        })
        .collect::<Result<_, _>>()
        .unwrap();

    for guest_path in std::fs::read_dir("../../guest/")
        .unwrap()
        .filter_map(Result::ok)
        .map(|de| de.path())
        .filter(|de| de.is_dir())
    {
        // HACK: Build wit files ahead of time so that we don't need to use a macro in the guest code.
        if guest_path.file_name().unwrap_or_default() == "rust" {
            use wit_bindgen_core::{wit_parser, Files, Generator};
            use wit_parser::Interface;

            let mut generator = wit_bindgen_gen_guest_rust::RustWasm::new();

            fn find_file<'a>(files: &'a [File], name: &str) -> &'a File {
                files
                    .iter()
                    .find(|f| {
                        f.absolute_path
                            .file_name()
                            .and_then(|p| p.to_str())
                            .unwrap_or_default()
                            .starts_with(name)
                    })
                    .unwrap()
            }
            let host_file = find_file(&files, "host.wit");
            let guest_file = find_file(&files, "guest.wit");
            let interface_version = find_file(&files, "INTERFACE_VERSION");

            let imports = Interface::parse_file(host_file.absolute_path.as_path()).unwrap();
            let exports = Interface::parse_file(guest_file.absolute_path.as_path()).unwrap();

            let mut files = Files::default();
            generator.generate_all(&[imports], &[exports], &mut files);

            if let Some((filename, contents)) = files.iter().next() {
                let mut contents = std::str::from_utf8(contents)
                    .unwrap()
                    // temp ugly hack as our version of wit-bindgen does not expose these
                    .replace(
                        "mod host {",
                        "#[allow(missing_docs)] pub mod host { use super::wit_bindgen_guest_rust;",
                    )
                    .replace("mod guest {", "#[allow(missing_docs)] pub mod guest {");

                contents += &format!(
                    "#[allow(missing_docs)] pub const INTERFACE_VERSION: u32 = {};",
                    interface_version.contents.trim()
                );

                std::fs::write(
                    guest_path
                        .join("api")
                        .join("src")
                        .join("internal")
                        .join(filename),
                    contents,
                )
                .unwrap();
            };
        } else {
            copy_files(&guest_path, &files, &working_dir);
        }
    }
}

fn copy_files(guest_path: &Path, files: &[File], working_dir: &Path) {
    let target_wit_dir = guest_path.join("api").join("wit");
    std::fs::create_dir_all(&target_wit_dir).unwrap();

    for file in files {
        let filename = file
            .absolute_path
            .file_name()
            .and_then(|p| p.to_str())
            .unwrap();

        let target_path =
            ambient_std::path::normalize(&working_dir.join(target_wit_dir.join(filename)));

        let absolute_path_relative_to_common = {
            let mut target_path_it = target_path.iter();

            file.absolute_path
                .clone()
                .iter()
                .skip_while(|segment| {
                    // do a case-insensitive compare to avoid issues on Windows with rust-analyzer
                    // where the disk letter may be different case
                    target_path_it
                        .next()
                        .map(|s| s.eq_ignore_ascii_case(segment))
                        .unwrap_or(false)
                })
                .map(|segment| segment.to_string_lossy())
                .collect::<Vec<_>>()
                .join("/")
        };

        std::fs::write(
            target_path,
            format!(
                "/* This file was copied from {:?}. Do not edit it directly. */\n{}",
                absolute_path_relative_to_common, file.contents
            ),
        )
        .unwrap();
    }
}
