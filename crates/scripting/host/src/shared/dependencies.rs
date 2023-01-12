//! Created through perusal of
//!  - lib.rs
//!  - blessed.rs
//!  - arewegameyet.rs

use once_cell::sync::Lazy;
use std::collections::HashSet;

const RUST_PATTERNS: &[&str] = &[
    "thiserror",
    "once_cell",
    "bytes",
    "miette",
    "color-eyre",
    "itertools",
    "pin-project",
    "ordered-float",
    "snafu",
    "anyhow",
];

const DATA_STRUCTURES: &[&str] = &[
    "hashbrown",
    "phf",
    "indexmap",
    "half",
    "bitvec",
    "uint",
    "ndarray",
    "smallvec",
    "arrayvec",
    "tinyvec",
    "bitflags",
    "bvh",
];

const ALGORITHMS: &[&str] = &["rand", "fastrand"];

const DEBUGGING: &[&str] = &["log", "tracing"];

const PROCEDURAL_MACROS: &[&str] = &["syn", "darling", "quote", "strum", "paste"];

const ENCODING_DATA: &[&str] = &[
    "serde",
    "serde_json",
    "serde_with",
    "base64",
    "bincode",
    "bson",
    "serde_yaml",
    "bytemuck",
];

const TEXT_PROCESSING: &[&str] = &["regex", "textwrap", "indoc", "fancy_regex"];

const ASYNCHRONOUS: &[&str] = &["futures", "async-trait"];

const MATH: &[&str] = &[
    "rust_decimal",
    "num-traits",
    "nalgebra",
    "num-rational",
    "num",
    "noise",
];

const PARSER_TOOLING: &[&str] = &["nom", "pest", "chumsky"];

const DATE_AND_TIME: &[&str] = &["chrono-tz", "time", "chrono", "fake"];

const VALUE_FORMATTING: &[&str] = &["itoa", "humansize", "ryu", "chrono-humanize"];

const TEMPLATE_ENGINE: &[&str] = &["tera", "handlebars", "askama", "liquid"];

const IMAGES: &[&str] = &["image"];

const CACHING: &[&str] = &["cached", "lru", "string_cache", "internment"];

const MISC: &[&str] = &["uuid", "palette"];

const ANIMATION: &[&str] = &["pareen", "natura", "keyframe", "splines"];

const AI: &[&str] = &["pathfinding"];

static WHITELIST: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from_iter(
        [
            RUST_PATTERNS,
            DATA_STRUCTURES,
            ALGORITHMS,
            DEBUGGING,
            PROCEDURAL_MACROS,
            ENCODING_DATA,
            TEXT_PROCESSING,
            ASYNCHRONOUS,
            MATH,
            PARSER_TOOLING,
            DATE_AND_TIME,
            VALUE_FORMATTING,
            TEMPLATE_ENGINE,
            IMAGES,
            CACHING,
            MISC,
            ANIMATION,
            AI,
        ]
        .iter()
        .flat_map(|s| s.iter().copied()),
    )
});

pub fn merge_cargo_toml(
    scripting_interfaces: &[&str],
    existing_file: &str,
    new_file: &str,
) -> anyhow::Result<String> {
    if new_file.is_empty() {
        anyhow::bail!("cannot merge empty Cargo.toml");
    }

    let mut existing_manifest = cargo_toml::Manifest::from_str(existing_file)?;
    let new_manifest = cargo_toml::Manifest::from_str(new_file)?;

    fn merge_dependencies(
        scripting_interfaces: &[&str],
        new: &cargo_toml::DepsSet,
    ) -> anyhow::Result<cargo_toml::DepsSet> {
        for dep in new.keys() {
            if !WHITELIST.contains(dep.as_str()) && !scripting_interfaces.contains(&dep.as_str()) {
                anyhow::bail!("package `{dep}` is not in the dependency whitelist");
            }
        }

        Ok(new.clone())
    }

    existing_manifest.dependencies =
        merge_dependencies(scripting_interfaces, &new_manifest.dependencies)?;
    existing_manifest.build_dependencies =
        merge_dependencies(scripting_interfaces, &new_manifest.build_dependencies)?;
    existing_manifest.dev_dependencies =
        merge_dependencies(scripting_interfaces, &new_manifest.dev_dependencies)?;

    Ok(toml::to_string(&existing_manifest)?)
}

#[cfg(test)]
mod tests {
    use super::merge_cargo_toml;

    const SCRIPTING_INTERFACES: &[&str] = &["elements_scripting_interface"];

    const DEFAULT_FILE: &str = indoc::indoc! {r#"
        [package]
        edition = "2021"
        name = "test-module"
        description = "a cool description"
        version = "0.1.0"

        [lib]
        crate-type = ["cdylib"]

        [dependencies]
        elements_scripting_interface = {path = "../../elements_scripting_interface"}
    "#};

    #[test]
    fn cannot_merge_cargo_toml_with_empty_file() {
        assert_eq!(
            merge_cargo_toml(SCRIPTING_INTERFACES, DEFAULT_FILE, "")
                .err()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            "cannot merge empty Cargo.toml"
        );
    }

    #[test]
    fn can_merge_with_no_changes() {
        assert_eq!(
            merge_cargo_toml(SCRIPTING_INTERFACES, DEFAULT_FILE, DEFAULT_FILE).unwrap_or_default(),
            indoc::indoc! {r#"
                [package]
                name = "test-module"
                edition = "2021"
                version = "0.1.0"
                description = "a cool description"
                [dependencies.elements_scripting_interface]
                path = "../../elements_scripting_interface"
                features = []

                [lib]
                crate-type = ["cdylib"]
                required-features = []
            "#}
        );
    }

    #[test]
    fn can_merge_with_non_dependency_changes() {
        const NEW_FILE: &str = indoc::indoc! {r#"
            [package]
            edition = "2021"
            name = "test-module hey"
            description = "a cool description whoa!"
            version = "0.2.0"

            [lib]
            crate-type = ["cdylib"]
            ignored-key = "okay!"

            [dependencies]
            elements_scripting_interface = {path = "../../elements_scripting_interface"}
        "#};

        assert_eq!(
            merge_cargo_toml(SCRIPTING_INTERFACES, DEFAULT_FILE, NEW_FILE).unwrap_or_default(),
            indoc::indoc! {r#"
                [package]
                name = "test-module"
                edition = "2021"
                version = "0.1.0"
                description = "a cool description"
                [dependencies.elements_scripting_interface]
                path = "../../elements_scripting_interface"
                features = []

                [lib]
                crate-type = ["cdylib"]
                required-features = []
            "#}
        );
    }

    #[test]
    fn can_merge_with_indexmap() {
        const NEW_FILE: &str = indoc::indoc! {r#"
            [package]
            edition = "2021"
            name = "test-module"
            description = "a cool description"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            elements_scripting_interface = {path = "../../elements_scripting_interface"}
            indexmap = "1.9.2"
        "#};

        assert_eq!(
            merge_cargo_toml(SCRIPTING_INTERFACES, DEFAULT_FILE, NEW_FILE).unwrap_or_default(),
            indoc::indoc! {r#"
                [package]
                name = "test-module"
                edition = "2021"
                version = "0.1.0"
                description = "a cool description"

                [dependencies]
                indexmap = "1.9.2"

                [dependencies.elements_scripting_interface]
                path = "../../elements_scripting_interface"
                features = []

                [lib]
                crate-type = ["cdylib"]
                required-features = []
            "#}
        );
    }

    #[test]
    fn cannot_merge_with_malicious_package() {
        const NEW_FILE: &str = indoc::indoc! {r#"
            [package]
            edition = "2021"
            name = "test-module"
            description = "a cool description"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            elements_scripting_interface = {path = "../../elements_scripting_interface"}
            malicious-package = "42.0.0"
        "#};

        assert_eq!(
            merge_cargo_toml(SCRIPTING_INTERFACES, DEFAULT_FILE, NEW_FILE)
                .err()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            "package `malicious-package` is not in the dependency whitelist"
        );
    }
}
