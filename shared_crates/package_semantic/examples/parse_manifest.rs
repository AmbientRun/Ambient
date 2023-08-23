use std::path::{Path, PathBuf};

use ambient_package_semantic::{Printer, RetrievableFile, Semantic};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut semantic = Semantic::new(false).await?;
    let args: Vec<_> = std::env::args().collect();
    let target = args.get(1).expect("path or 'all' as first arg");
    let should_resolve = !args.get(2).is_some_and(|s| s == "--no-resolve");

    let paths = if target == "all" {
        all_examples()?
    } else {
        vec![PathBuf::from(target)]
    };

    for path in paths {
        anyhow::ensure!(path.is_absolute(), "{path:?} must be absolute");
        semantic
            .add_package(RetrievableFile::Path(path.join("ambient.toml")))
            .await?;
    }

    let mut printer = Printer::new();
    if should_resolve {
        semantic.resolve()?;
    }
    printer.print(&semantic)?;

    Ok(())
}

// Copied from campfire
fn all_examples() -> anyhow::Result<Vec<PathBuf>> {
    let mut examples = Vec::new();

    let wd = std::env::current_dir()?;
    for guest in all_directories_in(Path::new("guest"))? {
        for category_path in all_directories_in(&guest.join("examples"))? {
            for example_path in all_directories_in(&category_path)? {
                examples.push(wd.join(example_path));
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
