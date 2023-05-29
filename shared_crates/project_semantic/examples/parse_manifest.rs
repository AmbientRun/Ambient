use std::path::{Path, PathBuf};

use ambient_project_semantic::{
    Attribute, Component, Concept, FileProvider, Item, ItemMap, Message, ResolvableItemId, Scope,
    Semantic, Type, TypeInner,
};

pub fn main() -> anyhow::Result<()> {
    const SCHEMA_PATH: &str = "shared_crates/schema/src";

    struct DiskFileProvider(PathBuf);
    impl FileProvider for DiskFileProvider {
        fn get(&self, path: &Path) -> std::io::Result<String> {
            std::fs::read_to_string(self.0.join(path))
        }

        fn full_path(&self, path: &Path) -> PathBuf {
            self.0.join(path)
        }
    }

    let ambient_toml = Path::new("ambient.toml");

    let mut semantic = Semantic::new()?;
    semantic.add_file(
        ambient_toml,
        &DiskFileProvider(PathBuf::from(SCHEMA_PATH)),
        true,
    )?;

    if let Some(project_path) = std::env::args().nth(1) {
        if project_path == "all" {
            for path in all_examples()? {
                let file_provider = DiskFileProvider(path);
                semantic.add_file(ambient_toml, &file_provider, false)?;
            }
        } else {
            let file_provider = DiskFileProvider(project_path.into());
            semantic.add_file(ambient_toml, &file_provider, false)?;
        }
    }

    let mut printer = Printer { indent: 0 };
    semantic.resolve()?;
    printer.print(&semantic)?;

    Ok(())
}

struct Printer {
    indent: usize,
}
impl Printer {
    fn print(&mut self, semantic: &Semantic) -> anyhow::Result<()> {
        let items = &semantic.items;
        self.print_scope(items, &*items.get(semantic.root_scope)?)?;
        for id in semantic.organizations.values() {
            self.print_scope(items, &*items.get(*id)?)?;
        }
        Ok(())
    }

    fn print_scope(&mut self, items: &ItemMap, scope: &Scope) -> anyhow::Result<()> {
        for id in scope.components.values() {
            self.print_component(items, &*items.get(*id)?)?;
        }

        for id in scope.concepts.values() {
            self.print_concept(items, &*items.get(*id)?)?;
        }

        for id in scope.messages.values() {
            self.print_message(items, &*items.get(*id)?)?;
        }

        for id in scope.types.values() {
            self.print_type(items, &*items.get(*id)?)?;
        }

        for id in scope.attributes.values() {
            self.print_attribute(items, &*items.get(*id)?)?;
        }

        for (_, id) in scope.scopes.values() {
            self.print_scope(items, &*items.get(*id)?)?;
        }

        Ok(())
    }

    fn print_component(&mut self, items: &ItemMap, component: &Component) -> anyhow::Result<()> {
        self.print_indent();
        println!("{}", fully_qualified_path(items, component)?);

        self.with_indent(|p| {
            p.print_indent();
            println!("name: {:?}", component.name.as_deref().unwrap_or_default());

            p.print_indent();
            println!(
                "description: {:?}",
                component.description.as_deref().unwrap_or_default()
            );

            p.print_indent();
            println!("type: {}", write_resolvable_id(items, &component.type_)?);

            p.print_indent();
            println!("attributes:");
            p.with_indent(|p| {
                for attribute in &component.attributes {
                    p.print_indent();
                    println!("{}", write_resolvable_id(items, attribute)?);
                }
                Ok(())
            })?;

            p.print_indent();
            println!("default: {:?}", component.default);

            Ok(())
        })
    }

    fn print_concept(&mut self, items: &ItemMap, concept: &Concept) -> anyhow::Result<()> {
        self.print_indent();
        println!("{}", fully_qualified_path(items, concept)?);

        self.with_indent(|p| {
            p.print_indent();
            println!("name: {:?}", concept.name.as_deref().unwrap_or_default());

            p.print_indent();
            println!(
                "description: {:?}",
                concept.description.as_deref().unwrap_or_default()
            );

            p.print_indent();
            print!("extends:");
            for extend in &concept.extends {
                print!("{} ", write_resolvable_id(items, extend)?);
            }
            println!();

            p.print_indent();
            println!("components:");

            p.with_indent(|p| {
                for (component, value) in concept.components.iter() {
                    p.print_indent();
                    println!("{}: {:?}", write_resolvable_id(items, component)?, value,);
                }

                Ok(())
            })
        })
    }

    fn print_message(&mut self, items: &ItemMap, message: &Message) -> anyhow::Result<()> {
        self.print_indent();
        println!("{}", fully_qualified_path(items, message)?);

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
                    println!("{}: {}", id, write_resolvable_id(items, ty)?);
                }

                Ok(())
            })
        })
    }

    fn print_type(&mut self, items: &ItemMap, type_: &Type) -> anyhow::Result<()> {
        self.print_indent();
        println!("{}", fully_qualified_path(items, type_)?,);
        if let TypeInner::Enum(e) = &type_.inner {
            self.with_indent(|p| {
                for (name, description) in &e.members {
                    p.print_indent();
                    print!("{name}: {description}");
                    println!();
                }
                Ok(())
            })?;
        }
        Ok(())
    }

    fn print_attribute(&mut self, items: &ItemMap, attribute: &Attribute) -> anyhow::Result<()> {
        self.print_indent();
        println!("{}", fully_qualified_path(items, attribute)?);
        Ok(())
    }

    fn print_indent(&self) {
        for _ in 0..self.indent {
            print!("  ");
        }
    }

    fn with_indent(
        &mut self,
        f: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.indent += 1;
        f(self)?;
        self.indent -= 1;
        Ok(())
    }
}

fn write_resolvable_id<T: Item>(
    items: &ItemMap,
    r: &ResolvableItemId<T>,
) -> anyhow::Result<String> {
    Ok(match r {
        ResolvableItemId::Unresolved(unresolved) => format!("unresolved({:?})", unresolved),
        ResolvableItemId::Resolved(resolved) => {
            format!("{}", fully_qualified_path(items, &*items.get(*resolved)?)?)
        }
    })
}

fn fully_qualified_path<T: Item>(items: &ItemMap, item: &T) -> anyhow::Result<String> {
    let data = item.data();
    let mut path = vec![data.id.to_string()];
    let mut parent_id = data.parent_id;
    while let Some(this_parent_id) = parent_id {
        let parent = items.get(this_parent_id)?;
        let id = parent.data().id.to_string();
        if !id.is_empty() {
            path.push(id);
        }
        parent_id = parent.data().parent_id;
    }
    path.reverse();
    Ok(format!(
        "{}:{}{}",
        T::TYPE.to_string().to_lowercase(),
        path.join("/"),
        if data.is_ambient { " [A]" } else { "" }
    ))
}

// Copied from campfire
fn all_examples() -> anyhow::Result<Vec<PathBuf>> {
    let mut examples = Vec::new();

    for guest in all_directories_in(Path::new("guest"))? {
        for category_path in all_directories_in(&guest.join("examples"))? {
            for example_path in all_directories_in(&category_path)? {
                examples.push(example_path);
            }
        }
    }

    Ok(examples)
}

fn all_directories_in(path: &Path) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    Ok(std::fs::read_dir(path)?
        .filter_map(Result::ok)
        .map(|de| de.path())
        .filter(|p| p.is_dir()))
}
