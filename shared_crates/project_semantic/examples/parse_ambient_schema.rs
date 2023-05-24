use std::{fmt::Display, path::Path};

use ambient_project_semantic::{
    Attribute, Component, Concept, FileProvider, Item, Message, ResolvableItemId, Scope, Semantic,
    Type,
};

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
    semantic.resolve().unwrap();
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

        for id in scope.components.values() {
            self.with_indent(|p| {
                p.print_component(semantic, semantic.items.get_without_resolve(*id).unwrap())
            });
        }

        for id in scope.concepts.values() {
            self.with_indent(|p| {
                p.print_concept(semantic, semantic.items.get_without_resolve(*id).unwrap())
            });
        }

        for id in scope.messages.values() {
            self.with_indent(|p| {
                p.print_message(semantic, semantic.items.get_without_resolve(*id).unwrap())
            });
        }

        for id in scope.types.values() {
            self.with_indent(|p| {
                p.print_type(semantic, semantic.items.get_without_resolve(*id).unwrap())
            });
        }

        for id in scope.attributes.values() {
            self.with_indent(|p| {
                p.print_attribute(semantic, semantic.items.get_without_resolve(*id).unwrap())
            });
        }
    }

    fn print_component(&mut self, semantic: &Semantic, component: &Component) {
        self.print_indent();
        println!("component({}): ", component.id);

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
                write_resolvable_id(semantic, &component.type_, |t| t.to_string(semantic))
            );

            p.print_indent();
            print!("attributes: ");
            for attribute in &component.attributes {
                print!(
                    "{} ",
                    write_resolvable_id(semantic, attribute, |attribute| attribute.id.clone())
                );
            }
            println!();

            p.print_indent();
            println!("default: {:?}", component.default);
        });
    }

    fn print_concept(&mut self, semantic: &Semantic, concept: &Concept) {
        self.print_indent();
        println!("concept({}): ", concept.id);

        self.with_indent(|p| {
            p.print_indent();
            println!("name: {:?}", concept.name.as_deref().unwrap_or_default());

            p.print_indent();
            println!(
                "description: {:?}",
                concept.description.as_deref().unwrap_or_default()
            );

            p.print_indent();
            print!("extends: ");
            for extend in &concept.extends {
                print!(
                    "{} ",
                    write_resolvable_id(semantic, extend, |extend| extend.id.clone())
                );
            }
            println!();

            p.print_indent();
            println!("components:");

            p.with_indent(|p| {
                for (component, value) in concept.components.iter() {
                    p.print_indent();
                    println!(
                        "{}: {:?}",
                        write_resolvable_id(semantic, component, |component| component.id.clone()),
                        value,
                    );
                }
            });
        });
    }

    fn print_message(&mut self, semantic: &Semantic, message: &Message) {
        self.print_indent();
        println!("message({}): ", message.id);

        self.with_indent(|p| {
            p.print_indent();
            println!(
                "description: {:?}",
                message.description.as_deref().unwrap_or_default()
            );

            p.print_indent();
            println!("fields:");

            p.with_indent(|p| {
                for (id, ty) in message.fields.iter() {
                    p.print_indent();
                    println!(
                        "{}: {}",
                        id,
                        write_resolvable_id(semantic, ty, |ty| ty.to_string(semantic)),
                    );
                }
            });
        });
    }

    fn print_type(&mut self, semantic: &Semantic, type_: &Type) {
        self.print_indent();
        println!("type: {}", type_.to_string(semantic));
        if let Type::Enum(e) = type_ {
            self.with_indent(|p| {
                for member in &e.members {
                    p.print_indent();
                    print!("{}", member.name);
                    if let Some(description) = &member.description {
                        print!(": {description}");
                    }
                    println!();
                }
            });
        }
    }

    fn print_attribute(&mut self, _semantic: &Semantic, attribute: &Attribute) {
        self.print_indent();
        println!("attribute({}): ", attribute.id);
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

fn write_resolvable_id<T: Item, D: Display>(
    semantic: &Semantic,
    r: &ResolvableItemId<T>,
    extractor: impl FnOnce(&T) -> D,
) -> String {
    match r {
        ResolvableItemId::Unresolved(unresolved) => format!("unresolved({:?})", unresolved),
        ResolvableItemId::Resolved(resolved) => {
            format!(
                "{}",
                extractor(semantic.items.get_without_resolve(*resolved).unwrap())
            )
        }
    }
}
