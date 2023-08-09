use std::path::{Path, PathBuf};

use ambient_project_semantic::{Printer, Semantic};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut semantic = Semantic::new().await?;

    let paths = {
        let arg = std::env::args().nth(1).expect("path or 'all' as first arg");
        if arg == "all" {
            all_examples()?
        } else {
            vec![PathBuf::from(arg)]
        }
    };
    for path in paths {
        semantic.add_ember(&path).await?;
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
