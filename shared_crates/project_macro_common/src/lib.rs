extern crate proc_macro;

mod api_project;
pub use api_project::*;

pub const MANIFEST: &str = include_str!("../ambient.toml");
