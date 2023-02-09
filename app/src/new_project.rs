use std::path::Path;

use anyhow::Context;
use convert_case::Casing;
use elements_project::Identifier;
use indoc::indoc;

pub(crate) fn new_project(project_path: &Path, name: Option<&str>) -> anyhow::Result<()> {
    let name = name
        .map(anyhow::Ok)
        .unwrap_or_else(|| project_path.file_name().and_then(|s| s.to_str()).context("project path has no terminating segment"))?;

    let id = name.to_case(convert_case::Case::Snake);
    let id = Identifier::new(id).map_err(anyhow::Error::msg)?;

    let dot_cargo = project_path.join(".cargo");
    let src = project_path.join("src");
    std::fs::create_dir_all(project_path).context("Failed to create project directory")?;
    std::fs::create_dir_all(&dot_cargo).context("Failed to create .cargo directory")?;
    std::fs::create_dir_all(&src).context("Failed to create src directory")?;

    std::fs::write(
        project_path.join("elements.toml"),
        indoc! {r#"
            [project]
            id = "{{id}}"
            name = "{{name}}"
            version = "0.0.1"
        "#}
        .replace("{{id}}", id.as_ref())
        .replace("{{name}}", name),
    )
    .context("Failed to create elements.toml")?;

    std::fs::write(
        project_path.join("Cargo.toml"),
        indoc! {r#"
            [package]
            name = "{{id}}"
            edition = "2021"
            version = "0.1.0"

            [dependencies]
            elements_scripting_interface = { path = "interfaces/elements_scripting_interface" }

            [lib]
            crate-type = ["cdylib"]
        "#}
        .replace("{{id}}", id.as_ref()),
    )
    .context("Failed to create Cargo.toml")?;

    std::fs::write(
        project_path.join(".gitignore"),
        indoc! {r#"
            */interfaces
            */.vscode
        "#},
    )
    .context("Failed to create .gitignore")?;

    std::fs::write(
        dot_cargo.join("config.toml"),
        indoc! {r#"
            [build]
            target = "wasm32-wasi"
        "#},
    )
    .context("Failed to create .cargo/config.toml")?;

    std::fs::write(
        src.join("lib.rs"),
        indoc! {r#"
            use elements_scripting_interface::*;

            #[main]
            pub async fn main() -> EventResult {
                loop {
                    println!("Hello, world! It is {}", time());
                    sleep(0.5).await;
                }

                EventOk
            }
    "#},
    )
    .context("Failed to create src/lib.rs")?;

    log::info!("Project {name} with id {id} created at {project_path:?}");

    Ok(())
}
