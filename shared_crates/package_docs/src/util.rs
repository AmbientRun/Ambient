use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use ambient_package_json as apj;

pub fn diff_paths<'a>(from: &'a Path, to: &'a Path) -> String {
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

pub fn path_to_item(
    manifest: &apj::Manifest,
    scope_id_to_package_id: &HashMap<String, apj::ItemId<apj::Package>>,
    item_id: &apj::ErasedItemId,
) -> String {
    let mut segments = vec![];

    let mut next = Some(item_id);
    while let Some(current) = next {
        let data = manifest.items.get(current).unwrap().item().data();
        segments.push(current);
        next = data.parent_id.as_ref().map(|v| &v.0);
    }

    if let &[id] = &segments[..] {
        if let Some(apj::ItemVariant::Scope(_)) = manifest.items.get(id) {
            if let Some(package_id) = scope_id_to_package_id.get(id) {
                return manifest.get(package_id).name.clone();
            }
        }
    }

    // Drop the root scope as it's likely an auto-generated ID if there's more than one segment
    segments.pop();
    segments.reverse();

    segments
        .into_iter()
        .map(|v| manifest.items.get(v).unwrap().item().data().id.clone())
        .collect::<Vec<_>>()
        .join("::")
}

pub fn item_url(
    manifest: &apj::Manifest,
    scope_id_to_package_id: &HashMap<String, apj::ItemId<apj::Package>>,
    from: &str,
    item_id: &str,
) -> tera::Result<String> {
    if let Some(package_id) = scope_id_to_package_id.get(item_id) {
        return item_url(manifest, scope_id_to_package_id, from, &package_id.0);
    }

    let item = manifest
        .items
        .get(item_id)
        .ok_or_else(|| tera::Error::msg("Item not found"))?;

    let mut segments = vec![];
    match item {
        apj::ItemVariant::Package(v) => {
            segments.push("index.html".to_string());
            segments.push(v.data.id.clone());
        }
        apj::ItemVariant::Scope(v) => {
            segments.push("index.html".to_string());
            segments.push(v.data.id.clone());
        }
        apj::ItemVariant::Component(v) => {
            segments.push(format!("{}.html", v.data.id));
            segments.push("components".to_string());
        }
        apj::ItemVariant::Concept(v) => {
            segments.push(format!("{}.html", v.data.id));
            segments.push("concepts".to_string());
        }
        apj::ItemVariant::Message(v) => {
            segments.push(format!("{}.html", v.data.id));
            segments.push("messages".to_string());
        }
        apj::ItemVariant::Type(v) => {
            segments.push(format!("{}.html", v.data.id));
            segments.push("types".to_string());
        }
        apj::ItemVariant::Attribute(v) => {
            segments.push(format!("{}.html", v.data.id));
            segments.push("attributes".to_string());
        }
    }

    // Push in all the segments.
    let mut last = None;
    let mut next = item.item().data().parent_id.as_ref();
    while let Some(current) = next {
        let scope = manifest.get(&current);
        last = Some(current);
        next = scope.data.parent_id.as_ref();

        // Skip the last one: that's a root or a package
        if next.is_some() {
            segments.push(scope.data.id.clone());
        }
    }

    // If the last one is a scope, push the package ID
    if let Some(root) = last {
        if let Some(package) = scope_id_to_package_id
            .get(&root.0)
            .map(|id| manifest.get(id))
        {
            segments.push(package.data.id.clone());
            segments.push("packages".to_string());
        }
    } else if matches!(item, apj::ItemVariant::Package(_)) {
        segments.push("packages".to_string());
    }

    segments.reverse();

    Ok(diff_paths(Path::new(&from), &PathBuf::from_iter(segments)))
}

pub fn get_arg<T: serde::de::DeserializeOwned>(
    args: &HashMap<String, tera::Value>,
    name: &str,
) -> tera::Result<T> {
    Ok(tera::from_value::<T>(
        args.get(name)
            .ok_or_else(|| tera::Error::msg(format!("Missing argument `{}`", name)))?
            .clone(),
    )?)
}
