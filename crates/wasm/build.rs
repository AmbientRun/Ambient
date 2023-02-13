use std::path::PathBuf;

/// Copies all files in the wit/ folder to all guests.
fn main() {
    let working_dir = std::env::current_dir().unwrap();

    let filenames_to_copy: Vec<_> = std::fs::read_dir("wit")
        .unwrap()
        .map(|r| r.map(|de| de.path()))
        .collect::<Result<_, _>>()
        .unwrap();

    struct File {
        absolute_path: PathBuf,
        contents: String,
    }

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
        let target_wit_dir = guest_path.join("api").join("wit");
        std::fs::create_dir_all(&target_wit_dir).unwrap();

        for file in &files {
            let filename = file
                .absolute_path
                .file_name()
                .and_then(|p| p.to_str())
                .unwrap();

            let target_path =
                kiwi_std::path::normalize(&working_dir.join(target_wit_dir.join(filename)));

            let absolute_path_relative_to_common = {
                let mut target_path_it = target_path.iter();

                file.absolute_path
                    .clone()
                    .iter()
                    .skip_while(|segment| target_path_it.next() == Some(segment))
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
}
