use std::{collections::HashMap, path::Path, sync::Arc};

use ambient_package_json as apj;
use tera::{Context, Tera};

mod util;

pub fn write(output_path: &Path, json_path: &Path, autoreload: bool) -> anyhow::Result<()> {
    let manifest: Arc<apj::Manifest> =
        Arc::new(serde_json::from_str(&std::fs::read_to_string(json_path)?)?);

    let mut packages = vec![];
    let mut scope_id_to_package_id = HashMap::new();
    for (id, package) in manifest.packages() {
        packages.push((id.clone(), package));
        scope_id_to_package_id.insert(package.scope_id.0.clone(), id.clone());
    }
    let scope_id_to_package_id = Arc::new(scope_id_to_package_id);

    let mut tera = Tera::default();
    tera.add_raw_templates([
        // Views
        ("views/base", include_str!("templates/views/base.html.j2")),
        (
            "views/package",
            include_str!("templates/views/package.html.j2"),
        ),
        (
            "views/component",
            include_str!("templates/views/component.html.j2"),
        ),
        (
            "views/concept",
            include_str!("templates/views/concept.html.j2"),
        ),
        ("views/scope", include_str!("templates/views/scope.html.j2")),
        (
            "views/redirect",
            include_str!("templates/views/redirect.html.j2"),
        ),
        // Partials
        (
            "partials/scope",
            include_str!("templates/partials/scope.html.j2"),
        ),
        (
            "partials/item_sidebar",
            include_str!("templates/partials/item_sidebar.html.j2"),
        ),
        (
            "partials/scope_sidebar",
            include_str!("templates/partials/scope_sidebar.html.j2"),
        ),
        (
            "partials/sidebar_package_heading",
            include_str!("templates/partials/sidebar_package_heading.html.j2"),
        ),
        (
            "partials/item_sidebar_package_heading",
            include_str!("templates/partials/item_sidebar_package_heading.html.j2"),
        ),
        // Misc
        ("macros", include_str!("templates/macros.html.j2")),
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
            let to = util::get_arg::<String>(args, "to")?;
            let from = util::get_arg::<String>(args, "from")?;

            Ok(tera::to_value(util::diff_paths(
                Path::new(&from),
                Path::new(&to),
            ))?)
        },
    );
    tera.register_function("item_url", {
        let manifest = manifest.clone();
        let scope_id_to_package_id = scope_id_to_package_id.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let from = util::get_arg::<String>(args, "from")?;
            let item_id = util::get_arg::<String>(args, "item_id")?;
            let path = util::item_url(
                manifest.as_ref(),
                scope_id_to_package_id.as_ref(),
                &from,
                &item_id,
            )?;
            Ok(tera::to_value(path)?)
        }
    });
    tera.register_function("item_path", {
        let manifest = manifest.clone();
        let scope_id_to_package_id = scope_id_to_package_id.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let item_id = util::get_arg::<String>(args, "item_id")?;

            Ok(tera::to_value(util::path_to_item(
                &manifest,
                scope_id_to_package_id.as_ref(),
                &item_id,
            ))?)
        }
    });
    tera.register_function("item_package_id", {
        let manifest = manifest.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let item_id = util::get_arg::<String>(args, "item_id")?;

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

            Ok(tera::to_value(package_id)?)
        }
    });
    tera.register_function("get_item", {
        let manifest = manifest.clone();
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let id = util::get_arg::<String>(args, "item_id")?;
            let item = manifest
                .items
                .get(&id)
                .ok_or_else(|| tera::Error::msg("Item not found"))?;

            Ok(tera::to_value(item)?)
        }
    });
    tera.register_function(
        "value_string",
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let value = util::get_arg::<apj::Value>(args, "value")?;
            Ok(tera::to_value(value.to_string())?)
        },
    );

    let style_css_path = output_path.join("style.css");
    std::fs::write(&style_css_path, include_str!("style.css"))?;

    let ctx = GenContext {
        tera: &tera,
        style_css_path: &style_css_path,
        output_path,
        manifest: manifest.as_ref(),
        autoreload,
    };

    ctx.write_scope(
        output_path,
        &manifest.root_scope_id,
        manifest.get(&manifest.root_scope_id),
        false,
    )?;

    let packages_dir = output_path.join("packages");
    std::fs::create_dir_all(&packages_dir)?;

    for (package_id, package) in &packages {
        ctx.write_package(&packages_dir, &packages, package_id, package)?;
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

#[derive(Copy, Clone)]
struct GenContext<'a> {
    tera: &'a Tera,
    output_path: &'a Path,
    style_css_path: &'a Path,
    manifest: &'a apj::Manifest,
    autoreload: bool,
}
impl GenContext<'_> {
    fn write_package(
        &self,
        packages_dir: &Path,
        packages: &[(apj::ItemId<apj::Package>, &apj::Package)],
        package_id: &apj::ItemId<apj::Package>,
        package: &apj::Package,
    ) -> anyhow::Result<()> {
        let package_dir = packages_dir.join(&package.data.id);
        std::fs::create_dir_all(&package_dir)?;

        let scope = self.manifest.get(&package.scope_id);
        self.write_scope(&package_dir, &package.scope_id, scope, false)?;

        let package_path = package_dir.join("index.html");

        let mut packages = packages.to_vec();
        packages.sort_by(|a, b| a.1.name.cmp(&b.1.name));

        let mut tera_ctx = self.tera_ctx(&package_path, Some((package, package_id)));
        tera_ctx.insert("packages", &packages);
        tera_ctx.insert("scope", scope);
        tera_ctx.insert("scope_id", &package.scope_id);

        std::fs::write(package_path, self.tera.render("views/package", &tera_ctx)?)?;

        Ok(())
    }

    fn write_scope(
        &self,
        output_dir: &Path,
        scope_id: &apj::ItemId<apj::Scope>,
        scope: &apj::Scope,
        generate_view: bool,
    ) -> anyhow::Result<()> {
        for (scope_name, scope_id) in &scope.scopes {
            let scope = self.manifest.get(scope_id);
            let scope_dir = output_dir.join(scope_name);
            std::fs::create_dir_all(&scope_dir)?;
            self.write_scope(&scope_dir, scope_id, scope, true)?;
        }

        if !scope.components.is_empty() {
            let components_dir = output_dir.join("components");
            std::fs::create_dir_all(&components_dir)?;
            for (_, component_id) in &scope.components {
                let component = self.manifest.get(component_id);
                self.write_item(&components_dir, "views/component", component_id, component)?;
            }
        }

        if !scope.concepts.is_empty() {
            let concepts_dir = output_dir.join("concepts");
            std::fs::create_dir_all(&concepts_dir)?;
            for (_, concept_id) in &scope.concepts {
                let concept = self.manifest.get(concept_id);
                self.write_item(&concepts_dir, "views/concept", concept_id, concept)?;
            }
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
            let tera_ctx = self.tera_ctx(&output_path, Some((scope, scope_id)));
            std::fs::write(output_path, self.tera.render("views/scope", &tera_ctx)?)?;
        }

        Ok(())
    }

    fn write_item<T: apj::Item + serde::Serialize>(
        &self,
        output_dir: &Path,
        view_name: &str,
        item_id: &apj::ItemId<T>,
        item: &T,
    ) -> anyhow::Result<()> {
        let item_path = output_dir.join(format!("{}.html", item.data().id));

        let tera_ctx = self.tera_ctx(&item_path, Some((item, item_id)));
        std::fs::write(item_path, self.tera.render(view_name, &tera_ctx)?)?;

        Ok(())
    }

    fn tera_ctx<T: apj::Item + serde::Serialize>(
        &self,
        page_path: &Path,
        item: Option<(&T, &apj::ItemId<T>)>,
    ) -> Context {
        let mut ctx = Context::new();
        ctx.insert(
            "style_css_path",
            &util::diff_paths(page_path, self.style_css_path),
        );
        ctx.insert("autoreload", &self.autoreload);
        ctx.insert("page_url", &util::diff_paths(self.output_path, page_path));
        if let Some((item, item_id)) = item {
            ctx.insert("item", item);
            ctx.insert("item_id", item_id);
        }

        ctx
    }
}
