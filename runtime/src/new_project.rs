use anyhow::Context;

pub(crate) fn new_project(name: &str) -> anyhow::Result<()> {
    let project_path = std::env::current_dir().context("Failed to get current dir")?.join(name);
    let dot_cargo = project_path.join(".cargo");
    let scripts_main = project_path.join("scripts").join("main");
    let scripts_main_src = scripts_main.join("src");
    std::fs::create_dir_all(&project_path).context("Failed to create project directory")?;
    std::fs::create_dir_all(&dot_cargo).context("Failed to create .cargo directory")?;
    std::fs::create_dir_all(&scripts_main_src).context("Failed to create scripts directory")?;

    std::fs::write(
        project_path.join("Cargo.toml"),
        r#"[workspace]
members = [
    "scripts/*",
    "interfaces/*"
]
"#,
    )
    .context("Failed to create Cargo.toml")?;

    std::fs::write(
        project_path.join(".gitignore"),
        r#"
*/interfaces
*/.vscode
*/scripts/*/src/components.rs
*/scripts/*/src/params.rs
*/rust-toolchain.toml
"#,
    )
    .context("Failed to create Cargo.toml")?;

    std::fs::write(
        dot_cargo.join("config.toml"),
        r#"
[build]
target = "wasm32-wasi"
"#,
    )
    .context("Failed to create .cargo/config.toml")?;

    std::fs::write(
        scripts_main.join("Cargo.toml"),
        r#"[package]
name = "main"
edition = "2021"
version = "0.1.0"
[dependencies.tilt_runtime_scripting_interface]
path = "../../interfaces/tilt_runtime_scripting_interface"
features = []

[lib]
crate-type = ["cdylib"]
required-features = []

"#,
    )
    .context("Failed to create scripts/main/Config.toml")?;

    std::fs::write(
        scripts_main_src.join("lib.rs"),
        r#"use tilt_runtime_scripting_interface::*;

pub mod components;
pub mod params;

#[main]
pub async fn main() -> EventResult {
    loop {
        println!("Hello, world! It is {}", time());
        sleep(0.5).await;
    }

    EventOk
}

"#,
    )
    .context("Failed to create scripts/main/src/lib.rs")?;

    Ok(())
}
