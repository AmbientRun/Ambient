use clap::Parser;

#[derive(Parser, Clone)]
pub enum Release {
    /// Changes the version of all crates to match the given version
    UpdateVersion {
        #[clap()]
        new_version: String,
    },
}

pub(crate) fn main(args: &Release) -> anyhow::Result<()> {
    match args {
        Release::UpdateVersion { new_version } => update_version(new_version),
    }
}

fn update_version(new_version: &str) -> anyhow::Result<()> {
    fn edit_toml(path: &str, f: impl Fn(&mut toml_edit::Document)) -> anyhow::Result<()> {
        let mut toml = std::fs::read_to_string(path)?.parse::<toml_edit::Document>()?;
        f(&mut toml);
        std::fs::write(path, toml.to_string())?;

        Ok(())
    }

    edit_toml("ambient.toml", |toml| {
        toml["project"]["version"] = toml_edit::value(new_version);
    })?;

    edit_toml("Cargo.toml", |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);
    })?;

    edit_toml("web/Cargo.toml", |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);
    })?;

    edit_toml("guest/rust/Cargo.toml", |toml| {
        toml["workspace"]["package"]["version"] = toml_edit::value(new_version);

        for (key, value) in toml["workspace"]["dependencies"]
            .as_table_like_mut()
            .expect("dependencies is not a table")
            .iter_mut()
        {
            if !key.starts_with("ambient_") {
                continue;
            }

            let Some(table) = value.as_table_like_mut() else { continue; };
            table.insert("version", toml_edit::value(new_version));
        }
    })?;

    // Update installing.md
    {
        const PREFIX: &str = "cargo install --git https://github.com/AmbientRun/Ambient.git --tag";

        let path = "docs/src/user/installing.md";
        let document = std::fs::read_to_string(path)?
            .lines()
            .map(|l| {
                if l.starts_with(PREFIX) {
                    format!("{PREFIX} v{new_version} ambient")
                } else {
                    l.to_string()
                }
            })
            // newline at the end
            .chain(std::iter::once("".to_string()))
            .collect::<Vec<String>>()
            .join("\n");

        std::fs::write(path, document)?;
    }

    Ok(())
}
