use std::path::Path;

use clap::Parser;

#[derive(Parser, Clone)]
pub struct Install {
    #[clap(short = 'r', long)]
    /// Git revision to install. If both this and `--git-tag` are specified, this takes precedence.
    /// If neither are specified, the repository on the local filesystem is used.
    git_revision: Option<String>,
    #[clap(short = 't', long)]
    /// Git tag to install. If both this and `--git-revision` are specified, `--git-revision` takes precedence.
    /// If neither are specified, the repository on the local filesystem is used.
    git_tag: Option<String>,
}

pub fn main(install: &Install) -> anyhow::Result<()> {
    const GIT_REPOSITORY: &str = "https://github.com/AmbientRun/Ambient.git";
    let git_args = vec!["--git", GIT_REPOSITORY, "ambient"];

    let (suffix, args) = Option::or(
        install
            .git_revision
            .as_deref()
            .map(|rev| (rev, [git_args.as_slice(), &["--rev", rev]].concat())),
        install
            .git_tag
            .as_deref()
            .map(|tag| (tag, [git_args.as_slice(), &["--tag", tag]].concat())),
    )
    .unwrap_or_else(|| ("", vec!["--path", "app"]));

    install_version(suffix, &args)
}

fn install_version(suffix: &str, args: &[&str]) -> anyhow::Result<()> {
    let target_name = if suffix.is_empty() {
        "ambient".to_string()
    } else {
        format!("ambient-{suffix}")
    };

    let install_root = Path::new("tmp");
    let target_path = home::cargo_home()?.join("bin").join(target_name);

    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "install",
        "--locked",
        "--force",
        "--root",
        install_root.to_str().unwrap(),
    ]);
    cmd.args(args);

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("`cargo install` failed with status {}", status);
    }

    std::fs::copy(install_root.join("bin").join("ambient"), &target_path)?;
    log::info!("Installed ambient to {}", target_path.display());

    Ok(())
}
