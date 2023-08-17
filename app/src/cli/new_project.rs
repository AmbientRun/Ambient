use std::path::Path;

use ambient_ember::SnakeCaseIdentifier;
use ambient_native_std::git_revision;
use anyhow::Context;
use convert_case::Casing;

use super::ProjectPath;

pub(crate) fn handle(
    project_path: &ProjectPath,
    name: Option<&str>,
    api_path: Option<&str>,
) -> anyhow::Result<()> {
    let Some(project_path) = &project_path.fs_path else {
        anyhow::bail!("Cannot create project in a remote directory.");
    };

    // Build the identifier.
    let project_path = if let Some(name) = name {
        project_path.join(name)
    } else {
        project_path.to_owned()
    };

    let name = project_path
        .file_name()
        .and_then(|s| s.to_str())
        .context("project path has no terminating segment")?;

    if project_path.is_dir() && std::fs::read_dir(&project_path)?.next().is_some() {
        anyhow::bail!("project path {project_path:?} is not empty");
    }

    let id = name.to_case(convert_case::Case::Snake);
    let id = SnakeCaseIdentifier::new(&id).map_err(anyhow::Error::msg)?;

    // Create the folders.
    let dot_cargo_path = project_path.join(".cargo");
    let dot_vscode_path = project_path.join(".vscode");
    let src_path = project_path.join("src");

    std::fs::create_dir_all(&project_path).context("Failed to create project directory")?;
    std::fs::create_dir_all(&dot_cargo_path).context("Failed to create .cargo directory")?;
    std::fs::create_dir_all(&dot_vscode_path).context("Failed to create .vscode directory")?;
    std::fs::create_dir_all(&src_path).context("Failed to create src directory")?;

    // Write the files to disk.
    let ambient_toml = include_str!("new_project_template/ambient.toml")
        .replace("{{id}}", id.as_str())
        .replace("{{name}}", name);

    let cargo_toml = {
        // Special-case creating an example in guest/rust/examples so that it "Just Works".
        let segments = project_path.iter().collect::<Vec<_>>();
        let (replacement, in_ambient_examples) = match segments
            .windows(3)
            .position(|w| w == ["guest", "rust", "examples"])
        {
            Some(i) => {
                let number_of_parents = segments.len() - i - 2;
                (
                    format!(
                        r#"ambient_api = {{ path = "{}api" }}"#,
                        "../".repeat(number_of_parents)
                    ),
                    true,
                )
            }
            None => (
                #[cfg(feature = "production")]
                format!("ambient_api = \"{}\"", env!("CARGO_PKG_VERSION")),
                #[cfg(not(feature = "production"))]
                {
                    if let Some(api_path) = api_path {
                        log::info!("Ambient path: {}", api_path);
                        format!("ambient_api = {{ path = {:?} }}", api_path)
                    } else if let Some(rev) = git_revision() {
                        log::info!("Ambient revision: {}", rev);
                        format!("ambient_api = {{ git = \"https://github.com/AmbientRun/Ambient.git\", rev = \"{}\" }}", rev)
                    } else {
                        let version = env!("CARGO_PKG_VERSION");
                        log::info!("Ambient version: {}", version);
                        format!("ambient_api = \"{}\"", version)
                    }
                },
                false,
            ),
        };

        let template_cargo_toml = include_str!("new_project_template/Cargo.toml");
        let mut template_cargo_toml = template_cargo_toml.replace("{{id}}", id.as_str()).replace(
            "ambient_api = { path = \"../../../../guest/rust/api\" }",
            &replacement,
        );

        if in_ambient_examples {
            template_cargo_toml = template_cargo_toml.replace(
                r#"version = "0.0.1""#,
                "rust-version = {workspace = true}\nversion = {workspace = true}",
            )
        }

        template_cargo_toml
    };

    macro_rules! include_template {
        ($path:expr) => {
            (
                Path::new($path),
                include_str!(concat!("new_project_template/", $path)),
            )
        };
    }

    let files_to_write = &[
        // root
        (Path::new("ambient.toml"), ambient_toml.as_str()),
        (Path::new("Cargo.toml"), cargo_toml.as_str()),
        include_template!(".gitignore"),
        include_template!("rust-toolchain.toml"),
        // .cargo
        include_template!(".cargo/config.toml"),
        // .vscode
        include_template!(".vscode/settings.json"),
        include_template!(".vscode/launch.json"),
        include_template!(".vscode/extensions.json"),
        // src
        include_template!("src/client.rs"),
        include_template!("src/server.rs"),
    ];

    for (filename, contents) in files_to_write {
        std::fs::write(project_path.join(filename), contents)
            .with_context(|| format!("failed to create {filename:?}"))?;
    }

    log::info!("Project \"{name}\" with id `{id}` created at {project_path:?}");

    Ok(())
}
