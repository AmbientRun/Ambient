use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ambient_package_json as apj;
use apj::Item;
use tera::{Context, Tera};

pub fn write(output_path: &Path, json_path: &Path, autoreload: bool) -> anyhow::Result<()> {
    let manifest: apj::Manifest = serde_json::from_str(&std::fs::read_to_string(json_path)?)?;
    let items = Arc::new(manifest.items.clone());
    let packages: HashMap<_, _> = manifest
        .items
        .iter()
        .filter_map(|item| Some((item.0.as_str(), apj::Package::from_item_variant(item.1)?)))
        .collect();

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
        let items = items.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let from = get_arg::<String>(args, "from")?;

            let package_id = match (
                get_arg::<String>(args, "package_id"),
                get_arg::<String>(args, "item_id"),
            ) {
                (Ok(package_id), Err(_)) => package_id,
                (Err(_), Ok(item_id)) => items
                    .get(&item_id)
                    .ok_or_else(|| tera::Error::msg("Item not found"))?
                    .item()
                    .data()
                    .id
                    .clone(),
                (Ok(_), Ok(_)) => {
                    return Err(tera::Error::msg(
                        "Only one of `package_id` and `item_id` can be specified",
                    ))
                }
                (Err(_), Err(_)) => {
                    return Err(tera::Error::msg(
                        "One of `package_id` and `item_id` must be specified",
                    ))
                }
            };

            Ok(tera::to_value(diff_paths(
                Path::new(&from),
                &PathBuf::from(format!("packages/{package_id}/index.html")),
            ))?)
        }
    });

    let style_css_path = output_path.join("style.css");
    std::fs::write(&style_css_path, include_str!("style.css"))?;

    let ctx = GenContext {
        tera: &tera,
        style_css_path: &style_css_path,
        output_path,
        manifest: &manifest,
        autoreload,
    };

    let packages_dir = output_path.join("packages");
    std::fs::create_dir_all(&packages_dir)?;

    for (_, package) in &packages {
        write_package(ctx, &packages_dir, &packages, package)?;
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
    fn tera_ctx(&self, page_path: &Path) -> Context {
        let mut ctx = Context::new();
        ctx.insert(
            "style_css_path",
            &diff_paths(page_path, self.style_css_path),
        );
        ctx.insert("autoreload", &self.autoreload);
        ctx.insert("page_url", &diff_paths(self.output_path, page_path));

        ctx
    }
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

fn write_package(
    ctx: GenContext,
    packages_dir: &Path,
    packages: &HashMap<&str, &apj::Package>,
    package: &apj::Package,
) -> anyhow::Result<()> {
    let package_dir = packages_dir.join(&package.data.id);
    std::fs::create_dir_all(&package_dir)?;

    let scope = ctx.manifest.get(&package.scope_id);
    write_scope(ctx, &package_dir, scope, false)?;

    let package_path = package_dir.join("index.html");

    let mut packages = packages
        .values()
        .map(|p| (&p.name, &p.data.id))
        .collect::<Vec<_>>();
    packages.sort_by(|a, b| a.0.cmp(b.0));

    let mut tera_ctx = ctx.tera_ctx(&package_path);
    tera_ctx.insert("package", package);
    tera_ctx.insert("packages", &packages);
    tera_ctx.insert("scope", scope);

    std::fs::write(package_path, ctx.tera.render("views/package", &tera_ctx)?)?;

    Ok(())
}

fn write_scope(
    ctx: GenContext,
    output_dir: &Path,
    scope: &apj::Scope,
    generate_view: bool,
) -> anyhow::Result<()> {
    for (scope_name, scope_id) in &scope.scopes {
        let scope = ctx.manifest.get(scope_id);
        let scope_dir = output_dir.join(scope_name);
        std::fs::create_dir_all(&scope_dir)?;
        write_scope(ctx, &scope_dir, scope, true)?;
    }

    if !scope.components.is_empty() {
        let components_dir = output_dir.join("components");
        std::fs::create_dir_all(&components_dir)?;
        for (_, component_id) in &scope.components {
            let component = ctx.manifest.get(component_id);
            write_component(ctx, &components_dir, component)?;
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
        let scope_path = output_dir.join("index.html");
        let mut tera_ctx = ctx.tera_ctx(&scope_path);
        tera_ctx.insert("scope", scope);

        std::fs::write(scope_path, ctx.tera.render("views/scope", &tera_ctx)?)?;
    }

    Ok(())
}

fn write_component(
    ctx: GenContext,
    output_dir: &Path,
    component: &apj::Component,
) -> anyhow::Result<()> {
    let component_path = output_dir.join(format!("{}.html", component.data.id));

    let mut tera_ctx = ctx.tera_ctx(&component_path);
    tera_ctx.insert("component", component);

    std::fs::write(
        component_path,
        ctx.tera.render("views/component", &tera_ctx)?,
    )?;

    Ok(())
}
