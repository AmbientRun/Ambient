use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ambient_package_json as apj;
use tera::{Context, Tera};

pub fn write(output_path: &Path, json_path: &Path, autoreload: bool) -> anyhow::Result<()> {
    let manifest: Arc<apj::Manifest> =
        Arc::new(serde_json::from_str(&std::fs::read_to_string(json_path)?)?);

    let mut packages = vec![];
    let mut scope_id_to_package_id = HashMap::new();

    for (id, package) in manifest.packages() {
        packages.push((id.clone(), package));
        scope_id_to_package_id.insert(package.scope_id.0.clone(), id.clone());
    }

    let mut tera = Tera::default();
    tera.add_raw_templates([
        ("views/base", include_str!("templates/views/base.html.j2")),
        (
            "views/package",
            include_str!("templates/views/package.html.j2"),
        ),
        (
            "views/component",
            include_str!("templates/views/component.html.j2"),
        ),
        ("views/scope", include_str!("templates/views/scope.html.j2")),
        (
            "views/redirect",
            include_str!("templates/views/redirect.html.j2"),
        ),
        (
            "partials/scope",
            include_str!("templates/partials/scope.html.j2"),
        ),
        (
            "partials/item_sidebar",
            include_str!("templates/partials/item_sidebar.html.j2"),
        ),
    ])?;
    tera.register_filter(
        "markdown",
        |value: &tera::Value, _: &_| -> tera::Result<tera::Value> {
            let value = tera::from_value::<String>(value.clone())?;
            let value = markdown::to_html(&value);
            Ok(tera::to_value(value)?)
        },
    );
    tera.register_function(
        "rel_path",
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let to = get_arg::<String>(args, "to")?;
            let from = get_arg::<String>(args, "from")?;

            Ok(tera::to_value(diff_paths(
                Path::new(&from),
                Path::new(&to),
            ))?)
        },
    );
    tera.register_function("package_url", {
        let manifest = manifest.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let from = get_arg::<String>(args, "from")?;

            let package_id = get_arg::<String>(args, "package_id")
                .or_else(|_: tera::Error| {
                    let item_id = get_arg::<String>(args, "item_id")?;
                    let item = manifest
                        .items
                        .get(&item_id)
                        .ok_or_else(|| tera::Error::msg("Item not found"))?
                        .item();

                    Ok(item.data().id.clone())
                })
                .or_else(|_: tera::Error| {
                    Err(tera::Error::msg(
                        "One of `package_id` or `item_id` must be specified",
                    ))
                })?;

            Ok(tera::to_value(diff_paths(
                Path::new(&from),
                &PathBuf::from(format!("packages/{package_id}/index.html")),
            ))?)
        }
    });
    tera.register_function("item_path", {
        let manifest = manifest.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let data = get_data_arg(args, &manifest)?;

            Ok(tera::to_value(path_to_item(&manifest, &data))?)
        }
    });
    tera.register_function("item_package", {
        let manifest = manifest.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let item_id = get_arg::<String>(args, "item_id")?;

            let mut next = &item_id;
            loop {
                let current = manifest.items.get(next).unwrap();
                if let Some(parent_id) = current.item().data().parent_id.as_ref() {
                    next = &parent_id.0;
                } else {
                    break;
                }
            }

            let package_id = scope_id_to_package_id.get(next).ok_or_else(|| {
                tera::Error::msg(format!("Could not find package for item `{}`", item_id))
            })?;

            Ok(tera::to_value(manifest.get(package_id))?)
        }
    });
    tera.register_function("get_item", {
        let manifest = manifest.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let id = get_arg::<String>(args, "id")?;
            let item = manifest
                .items
                .get(&id)
                .ok_or_else(|| tera::Error::msg("Item not found"))?;

            Ok(tera::to_value(item)?)
        }
    });

    let style_css_path = output_path.join("style.css");
    std::fs::write(&style_css_path, include_str!("style.css"))?;

    let ctx = GenContext {
        tera: &tera,
        style_css_path: &style_css_path,
        output_path,
        manifest: manifest.as_ref(),
        autoreload,
    };

    write_scope(
        ctx,
        output_path,
        &manifest.root_scope_id,
        manifest.get(&manifest.root_scope_id),
        false,
    )?;

    let packages_dir = output_path.join("packages");
    std::fs::create_dir_all(&packages_dir)?;

    for (package_id, package) in &packages {
        write_package(ctx, &packages_dir, &packages, package_id, package)?;
    }

    let index_path = output_path.join("index.html");
    let mut index_context = Context::new();
    index_context.insert(
        "redirect_url",
        &format!("./packages/{}/index.html", manifest.main_package().data.id),
    );
    std::fs::write(index_path, tera.render("views/redirect", &index_context)?)?;

    Ok(())
}

fn get_data_arg(
    args: &HashMap<String, serde_json::Value>,
    manifest: &Arc<apj::Manifest>,
) -> tera::Result<apj::ItemData> {
    let data = get_arg::<apj::ItemData>(args, "data")
        .or_else(|_: tera::Error| {
            let item_id = get_arg::<String>(args, "item_id")?;
            let item = manifest
                .items
                .get(&item_id)
                .ok_or_else(|| tera::Error::msg("Item not found"))?
                .item();

            Ok(item.data().clone())
        })
        .or_else(|_: tera::Error| {
            Err(tera::Error::msg(
                "One of `data` or `item_id` must be specified",
            ))
        })?;
    Ok(data)
}

fn get_arg<T: serde::de::DeserializeOwned>(
    args: &HashMap<String, tera::Value>,
    name: &str,
) -> tera::Result<T> {
    Ok(tera::from_value::<T>(
        args.get(name)
            .ok_or_else(|| tera::Error::msg(format!("Missing argument `{}`", name)))?
            .clone(),
    )?)
}

#[derive(Copy, Clone)]
struct GenContext<'a> {
    tera: &'a Tera,
    output_path: &'a Path,
    style_css_path: &'a Path,
    manifest: &'a apj::Manifest,
    autoreload: bool,
}
impl GenContext<'_> {
    fn tera_ctx<T: apj::Item + serde::Serialize>(
        &self,
        page_path: &Path,
        item: Option<(&T, &apj::ItemId<T>)>,
    ) -> Context {
        let mut ctx = Context::new();
        ctx.insert(
            "style_css_path",
            &diff_paths(page_path, self.style_css_path),
        );
        ctx.insert("autoreload", &self.autoreload);
        ctx.insert("page_url", &diff_paths(self.output_path, page_path));
        if let Some((item, item_id)) = item {
            ctx.insert("item", item);
            ctx.insert("item_id", item_id);
        }

        ctx
    }
}

fn write_package(
    ctx: GenContext,
    packages_dir: &Path,
    packages: &[(apj::ItemId<apj::Package>, &apj::Package)],
    package_id: &apj::ItemId<apj::Package>,
    package: &apj::Package,
) -> anyhow::Result<()> {
    let package_dir = packages_dir.join(&package.data.id);
    std::fs::create_dir_all(&package_dir)?;

    let scope = ctx.manifest.get(&package.scope_id);
    write_scope(ctx, &package_dir, &package.scope_id, scope, false)?;

    let package_path = package_dir.join("index.html");

    let mut packages = packages
        .iter()
        .map(|(_, p)| (&p.name, &p.data.id))
        .collect::<Vec<_>>();
    packages.sort_by(|a, b| a.0.cmp(b.0));

    let mut tera_ctx = ctx.tera_ctx(&package_path, Some((package, package_id)));
    tera_ctx.insert("package", package);
    tera_ctx.insert("packages", &packages);
    tera_ctx.insert("scope", scope);

    std::fs::write(package_path, ctx.tera.render("views/package", &tera_ctx)?)?;

    Ok(())
}

fn write_scope(
    ctx: GenContext,
    output_dir: &Path,
    scope_id: &apj::ItemId<apj::Scope>,
    scope: &apj::Scope,
    generate_view: bool,
) -> anyhow::Result<()> {
    for (scope_name, scope_id) in &scope.scopes {
        let scope = ctx.manifest.get(scope_id);
        let scope_dir = output_dir.join(scope_name);
        std::fs::create_dir_all(&scope_dir)?;
        write_scope(ctx, &scope_dir, scope_id, scope, true)?;
    }

    if !scope.components.is_empty() {
        let components_dir = output_dir.join("components");
        std::fs::create_dir_all(&components_dir)?;
        for (_, component_id) in &scope.components {
            let component = ctx.manifest.get(component_id);
            write_component(ctx, &components_dir, component_id, component)?;
        }
    }

    if !scope.concepts.is_empty() {
        let concepts_dir = output_dir.join("concepts");
        std::fs::create_dir_all(&concepts_dir)?;
    }

    if !scope.messages.is_empty() {
        let messages_dir = output_dir.join("messages");
        std::fs::create_dir_all(&messages_dir)?;
    }

    if !scope.types.is_empty() {
        let types_dir = output_dir.join("types");
        std::fs::create_dir_all(&types_dir)?;
    }

    if !scope.attributes.is_empty() {
        let attributes_dir = output_dir.join("attributes");
        std::fs::create_dir_all(&attributes_dir)?;
    }

    if generate_view {
        let output_path = output_dir.join("index.html");
        let mut tera_ctx = ctx.tera_ctx(&output_path, Some((scope, scope_id)));
        tera_ctx.insert("scope", scope);

        std::fs::write(output_path, ctx.tera.render("views/scope", &tera_ctx)?)?;
    }

    Ok(())
}

fn write_component(
    ctx: GenContext,
    output_dir: &Path,
    component_id: &apj::ItemId<apj::Component>,
    component: &apj::Component,
) -> anyhow::Result<()> {
    let component_path = output_dir.join(format!("{}.html", component.data.id));

    let tera_ctx = ctx.tera_ctx(&component_path, Some((component, component_id)));

    std::fs::write(
        component_path,
        ctx.tera.render("views/component", &tera_ctx)?,
    )?;

    Ok(())
}

fn diff_paths<'a>(from: &'a Path, to: &'a Path) -> String {
    pathdiff::diff_paths(
        to,
        if from.extension().is_some() {
            from.parent().unwrap()
        } else {
            from
        },
    )
    .unwrap()
    .to_string_lossy()
    .to_string()
}

fn path_to_item(manifest: &apj::Manifest, data: &apj::ItemData) -> String {
    let mut segments = vec![];

    let mut next = Some(data);
    while let Some(current) = next {
        segments.push(current.id.clone());
        next = current
            .parent_id
            .as_ref()
            .map(|id| manifest.get(id) as &dyn apj::Item)
            .map(|item| item.data());
    }
    // Drop the root scope as it's likely an auto-generated ID
    segments.pop();
    segments.reverse();

    segments.join("::")
}
