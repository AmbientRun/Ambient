use std::path::{Path, PathBuf};

use crate::Example;

pub(crate) fn main(ex: &Example) -> anyhow::Result<()> {
    match ex {
        Example::Clean => clean()?,
    }

    Ok(())
}

fn clean() -> anyhow::Result<()> {
    log::info!("Cleaning examples...");
    for example_path in all_examples()? {
        let build_path = example_path.join("build");
        if !build_path.exists() {
            continue;
        }

        std::fs::remove_dir_all(&build_path)?;
        log::info!("Removed build directory for {}.", example_path.display());
    }
    log::info!("Done cleaning examples.");
    Ok(())
}

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
        .into_iter()
        .filter_map(Result::ok)
        .map(|de| de.path())
        .filter(|p| p.is_dir()))
}
