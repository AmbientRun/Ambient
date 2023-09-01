use std::{fs::DirEntry, path::Path};

pub fn run_ambient(args: &[&str], release: bool) -> anyhow::Result<()> {
    // TODO: consider running other versions of Ambient
    let mut command = std::process::Command::new("cargo");
    command.arg("run");
    if release {
        command.arg("--release");
    }
    command.args(["-p", "ambient"]).args(args).spawn()?.wait()?;

    Ok(())
}

pub fn all_directories_in(path: &Path) -> anyhow::Result<impl Iterator<Item = DirEntry>> {
    Ok(std::fs::read_dir(path)?
        .filter_map(Result::ok)
        .filter(|p| p.path().is_dir()))
}
