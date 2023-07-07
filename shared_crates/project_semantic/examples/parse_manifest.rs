use std::path::{Path, PathBuf};

use ambient_project_semantic::{FileProvider, Printer, Semantic};

pub fn main() -> anyhow::Result<()> {
    const SCHEMA_PATH: &str = "shared_crates/schema/src";

    struct DiskFileProvider(PathBuf);
    impl FileProvider for DiskFileProvider {
        fn get(&self, path: &Path) -> std::io::Result<String> {
            std::fs::read_to_string(self.0.join(path))
        }

        fn full_path(&self, path: &Path) -> PathBuf {
            self.0.join(path)
        }
    }

    let ambient_toml = Path::new("ambient.toml");

    let mut semantic = Semantic::new()?;
    semantic.add_file(
        ambient_toml,
        &DiskFileProvider(PathBuf::from(SCHEMA_PATH)),
        true,
    )?;

    if let Some(project_path) = std::env::args().nth(1) {
        if project_path == "all" {
            for path in all_examples()? {
                let file_provider = DiskFileProvider(path);
                semantic.add_file(ambient_toml, &file_provider, false)?;
            }
        } else {
            let file_provider = DiskFileProvider(project_path.into());
            semantic.add_file(ambient_toml, &file_provider, false)?;
        }
    }

    let mut printer = Printer::new();
    semantic.resolve()?;
    printer.print(&semantic)?;

    Ok(())
}

// Copied from campfire
fn all_examples() -> anyhow::Result<Vec<PathBuf>> {
    let mut examples = Vec::new();

    for guest in all_directories_in(Path::new("guest"))? {
        for category_path in all_directories_in(&guest.join("examples"))? {
            for example_path in all_directories_in(&category_path)? {
                examples.push(example_path);
            }
        }
    }

    Ok(examples)
}

fn all_directories_in(path: &Path) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    Ok(std::fs::read_dir(path)?
        .filter_map(Result::ok)
        .map(|de| de.path())
        .filter(|p| p.is_dir()))
}
