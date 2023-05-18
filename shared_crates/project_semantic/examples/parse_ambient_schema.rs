use std::{fmt::Display, path::Path};

use ambient_project::Identifier;
use ambient_project_semantic::{Component, FileProvider, Item, ItemId, ItemRef, Scope, Semantic};

pub fn main() {
    const SCHEMA_PATH: &str = "shared_crates/schema/src";

    struct DiskFileProvider;
    impl FileProvider for DiskFileProvider {
        fn get(&self, filename: &str) -> std::io::Result<String> {
            std::fs::read_to_string(Path::new(SCHEMA_PATH).join(filename))
        }
    }

    let mut semantic = Semantic::new();
    semantic
        .add_file("ambient.toml", &DiskFileProvider)
        .unwrap();

    let mut printer = Printer { indent: 0 };

    println!("------ Pre-resolve:");
    printer.print(&semantic);

    semantic.resolve().unwrap();

    println!("------ Post-resolve:");
    printer.print(&semantic);
}

struct Printer {
    indent: usize,
}
impl Printer {
    fn print(&mut self, semantic: &Semantic) {
        for scope in semantic.scopes.values() {
            self.print_scope(semantic, scope);
        }
    }

    fn print_scope(&mut self, semantic: &Semantic, scope: &Scope) {
        self.print_indent();
        println!("{}: ", scope.id);
        for scope in scope.scopes.values() {
            self.with_indent(|p| p.print_scope(semantic, scope));
        }

        for (name, id) in scope.components.iter() {
            self.with_indent(|p| p.print_component(semantic, name, *id));
        }
    }

    fn print_component(&mut self, semantic: &Semantic, name: &Identifier, id: ItemId<Component>) {
        let component = semantic.items.get(id).unwrap();
        self.print_indent();
        println!("component({}): ", name);

        self.with_indent(|p| {
            p.print_indent();
            println!("name: {:?}", component.name.as_deref().unwrap_or_default());
            p.print_indent();
            println!(
                "description: {:?}",
                component.description.as_deref().unwrap_or_default()
            );
            p.print_indent();
            println!(
                "type: {}",
                write_ref(semantic, &component.type_, |t| t.to_string(semantic))
            );
            p.print_indent();
            print!("attributes: ");
            for attribute in &component.attributes {
                print!(
                    "{} ",
                    write_ref(semantic, attribute, |attribute| attribute.id.clone())
                );
            }
            println!();
            p.print_indent();
            println!("default: {:?}", component.default);
        });
    }

    fn print_indent(&self) {
        for _ in 0..self.indent {
            print!("  ");
        }
    }

    fn with_indent(&mut self, f: impl FnOnce(&mut Self)) {
        self.indent += 1;
        f(self);
        self.indent -= 1;
    }
}

fn write_ref<T: Item, D: Display>(
    semantic: &Semantic,
    r: &ItemRef<T>,
    extractor: impl FnOnce(&T) -> D,
) -> String {
    match r {
        ItemRef::Unresolved(unresolved) => format!("unresolved({:?})", unresolved),
        ItemRef::Resolved(resolved) => {
            format!("{}", extractor(semantic.items.get(*resolved).unwrap()))
        }
    }
}
