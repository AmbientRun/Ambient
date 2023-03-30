extern crate proc_macro;

use std::path::PathBuf;

use anyhow::Context;
use proc_macro::TokenStream;
use proc_macro2::Span;

mod main_macro;

/// Generates global components and other boilerplate for the API crate.
#[proc_macro]
pub fn api_project(_input: TokenStream) -> TokenStream {
    TokenStream::from(
        ambient_project_macro_common::implementation(
            (None, ambient_project_macro_common::MANIFEST.to_string()),
            ambient_project_macro_common::Context::Guest {
                api_path: syn::Path::from(syn::Ident::new("crate", Span::call_site())),
                fully_qualified_path: true,
            },
            true,
            true,
        )
        .unwrap(),
    )
}

/// Makes your `main()` function accessible to the WASM host, and generates `components` and `concept` modules for your project.
///
/// If you do not add this attribute to your `main()` function, your module will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    main_macro::main_impl(
        item.into(),
        read_file("ambient.toml".to_string()).expect("Failed to load ambient.toml"),
    )
    .unwrap()
    .into()
}

fn read_file(file_path: String) -> anyhow::Result<(Option<String>, String)> {
    let file_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("no manifest dir")?)
        .join(file_path);

    Ok((
        Some(format!("{}", file_path.display())),
        std::fs::read_to_string(&file_path)?,
    ))
}
