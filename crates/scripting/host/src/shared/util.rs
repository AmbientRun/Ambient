use super::script_module;
use elements_core::name;
use elements_ecs::{query, EntityId, World};
use indoc::indoc;
use std::collections::HashSet;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

pub fn write_files_to_directory(
    base_path: &Path,
    files: &[(PathBuf, String)],
) -> anyhow::Result<()> {
    let folders: HashSet<_> = files
        .iter()
        .map(|(p, _)| p)
        .filter_map(|k| k.parent().map(|p| p.to_owned()))
        .collect();

    for folder in folders {
        std::fs::create_dir_all(base_path.join(folder))?;
    }

    for (path, contents) in files {
        std::fs::write(base_path.join(path), contents)?;
    }
    Ok(())
}

pub fn all_module_names_sanitized(world: &World, include_disabled_modules: bool) -> Vec<String> {
    query(script_module())
        .iter(world, None)
        .filter_map(|(id, sm)| {
            (include_disabled_modules || sm.enabled).then(|| sanitize(&get_module_name(world, id)))
        })
        .collect()
}

pub fn write_workspace_files(
    workspace_path: &Path,
    script_module_sanitized_names: &[String],
    write_workspace_toml: bool,
) {
    let vscode_dir = workspace_path.join(".vscode");
    let mut workspace_files = vec![
        (
            workspace_path.join("rust-toolchain.toml"),
            indoc! {r#"
            [toolchain]
            targets = ["wasm32-wasi"]
            "#}
            .into(),
        ),
        (
            workspace_path.join(".cargo").join("config.toml"),
            indoc! {r#"
            [build]
            target = "wasm32-wasi"
            "#}
            .into(),
        ),
        (
            vscode_dir.join("extensions.json"),
            r#"{"recommendations": ["rust-lang.rust-analyzer"]}"#.into(),
        ),
        (
            vscode_dir.join("settings.json"),
            indoc! {r#"
            {
                "rust-analyzer.cargo.target": "wasm32-wasi"
            }
            "#}
            .into(),
        ),
    ];
    if write_workspace_toml {
        workspace_files.push((
            workspace_path.join("Cargo.toml"),
            format!("[workspace]\nmembers = {script_module_sanitized_names:?}"),
        ));
    }

    for (path, contents) in workspace_files {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }
}

pub fn remove_old_script_modules(scripts_dir: &Path, script_module_sanitized_names: &[String]) {
    // Remove all directories that are not within the current working set of modules.
    if let Ok(entries) = std::fs::read_dir(scripts_dir) {
        for path in entries
            .filter_map(Result::ok)
            .map(|de| de.path())
            .filter(|p| p.is_dir())
            .filter(|p| {
                let dir_name = p.file_name().unwrap_or_default().to_string_lossy();
                let should_be_kept = dir_name == "target"
                    || dir_name.starts_with('.')
                    || script_module_sanitized_names
                        .iter()
                        .any(|m| m.as_str() == dir_name);
                !should_be_kept
            })
        {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

pub fn sanitize<T: Display>(val: &T) -> String {
    val.to_string().replace(':', "_")
}

pub fn unsanitize<T: FromStr>(val: &str) -> anyhow::Result<T>
where
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    Ok(val.replace('_', ":").parse()?)
}

pub fn get_module_name(world: &World, id: EntityId) -> String {
    world.get_cloned(id, name()).unwrap_or(id.to_string())
}
