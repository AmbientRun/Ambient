use std::path::{Path, PathBuf};

struct File {
    relative_path: PathBuf,
    contents: String,
}

/// Copies all files in the wit/ folder to all guests, and generates wit ahead of time
/// for Rust guests.
fn main() {
    let working_dir = std::env::current_dir().unwrap().canonicalize().unwrap();
    // De-UNC the path.
    #[cfg(target_os = "windows")]
    let working_dir = dunce::simplified(&working_dir).to_owned();

    println!("cargo:rerun-if-changed=wit");
    let filenames_to_copy: Vec<_> = std::fs::read_dir("wit")
        .unwrap()
        .map(|r| r.map(|de| de.path()))
        .collect::<Result<_, _>>()
        .unwrap();

    fn load_files(working_dir: &Path, path: &Path) -> std::io::Result<Vec<File>> {
        let absolute_path = working_dir.join(path).canonicalize().unwrap();

        // De-UNC the path.
        #[cfg(target_os = "windows")]
        let absolute_path = dunce::simplified(&absolute_path).to_owned();

        if absolute_path.is_file() {
            Ok(vec![File {
                relative_path: absolute_path
                    .strip_prefix(working_dir)
                    .unwrap()
                    .strip_prefix("wit")
                    .unwrap()
                    .to_owned(),
                contents: std::fs::read_to_string(path)?,
            }])
        } else if absolute_path.is_dir() {
            let mut paths = vec![];
            for entry in std::fs::read_dir(path)? {
                let path = entry?.path();
                paths.extend(load_files(working_dir, &path)?);
            }
            Ok(paths)
        } else {
            panic!("Invalid path to copy: {:?}", absolute_path);
        }
    }

    let mut files = vec![];
    for filename in filenames_to_copy {
        files.extend(load_files(&working_dir, &filename).unwrap());
    }

    eprintln!("Assembling guest files");
    for guest_path in std::fs::read_dir("../../guest/")
        .unwrap()
        .filter_map(Result::ok)
        .map(|de| de.path())
        .filter(|de| de.is_dir())
    {
        eprintln!("Reading file in guest path: {guest_path:?}");
        // HACK: Build wit files ahead of time so that we don't need to use a macro in the guest code.
        if guest_path.file_name().unwrap_or_default() == "rust" {
            eprintln!("Reading rust wit");
            use wit_bindgen_core::{wit_parser::Resolve, Files};

            let mut generator = wit_bindgen_rust::Opts::default().build();
            let mut resolve = Resolve::new();
            let pkg = resolve.push_dir(Path::new("wit")).unwrap().0;

            let mut files = Files::default();
            let world = resolve.select_world(pkg, None).unwrap();
            generator.generate(&resolve, world, &mut files);

            for (filename, contents) in files.iter() {
                eprintln!("Writing file: {filename:?}");
                std::fs::write(
                    guest_path
                        .join("api_core")
                        .join("src")
                        .join("internal")
                        .join(filename),
                    contents,
                )
                .unwrap();
            }
        } else {
            copy_files(&guest_path, &files, &working_dir);
        }
    }
}

fn copy_files(guest_path: &Path, files: &[File], working_dir: &Path) {
    let target_wit_dir = guest_path.join("api").join("wit");

    for file in files {
        let target_path = ambient_std::path::normalize(
            &working_dir.join(target_wit_dir.join(&file.relative_path)),
        );

        std::fs::create_dir_all(target_path.parent().unwrap()).unwrap();
        std::fs::write(
            target_path,
            format!(
                "/* This file was automatically copied from the repository. Do not edit it directly. */\n{}",
                file.contents
            ),
        )
        .unwrap();
    }
}
