extern crate proc_macro;

use ambient_project_semantic::{FileProvider, ItemMap, Scope, Semantic};
use proc_macro2::TokenStream;
use quote::quote;
use std::path::{Path, PathBuf};

pub enum Context {
    Host,
    Guest {
        api_path: syn::Path,
        fully_qualified_path: bool,
    },
}

pub enum ManifestSource {
    Path(PathBuf),
    /// Does not support paths to other files
    String(String),
}

pub fn generate_code(
    // bool is whether or not it's ambient
    manifests: Vec<(ManifestSource, bool)>,
    context: Context,
) -> anyhow::Result<TokenStream> {
    struct DiskFileProvider(PathBuf);
    impl FileProvider for DiskFileProvider {
        fn get(&self, path: &Path) -> std::io::Result<String> {
            std::fs::read_to_string(self.0.join(path))
        }

        fn full_path(&self, path: &Path) -> PathBuf {
            self.0.join(path)
        }
    }

    struct StringFileProvider(String);
    impl FileProvider for StringFileProvider {
        fn get(&self, path: &Path) -> std::io::Result<String> {
            if path.to_string_lossy() == "ambient.toml" {
                Ok(self.0.clone())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "not found",
                ))
            }
        }

        fn full_path(&self, path: &Path) -> PathBuf {
            path.to_owned()
        }
    }

    let mut semantic = Semantic::new()?;
    for (manifest, ambient) in manifests {
        match manifest {
            ManifestSource::Path(path) => {
                semantic.add_file(
                    &path,
                    &DiskFileProvider(path.parent().unwrap().to_owned()),
                    ambient,
                )?;
            }
            ManifestSource::String(string) => {
                semantic.add_file(
                    Path::new("ambient.toml"),
                    &StringFileProvider(string),
                    ambient,
                )?;
            }
        }
    }

    let mut printer = ambient_project_semantic::Printer::new();
    semantic.resolve()?;
    printer.print(&semantic)?;

    let items = &semantic.items;
    let components = make_component_definitions(&items, &*items.get(semantic.root_scope)?)?;

    let output = quote! {
        /// Auto-generated component definitions. These come from `ambient.toml` in the root of the project.
        pub mod components {}
        /// Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept.
        ///
        /// They do not have any runtime representation outside of the components that compose them.
        pub mod concepts {}
        /// Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,
        /// and with other modules.
        pub mod messages {}
    };

    println!("{}", output.to_string());

    Ok(output)
}

fn make_component_definitions(items: &ItemMap, scope: &Scope) -> anyhow::Result<TokenStream> {
    Ok(quote! {})
}
