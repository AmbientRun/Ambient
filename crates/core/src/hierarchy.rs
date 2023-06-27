use std::collections::HashSet;

use ambient_ecs::{query, Component, ComponentValue, ECSError, EntityId, World};
use ambient_std::download_asset::AssetsCacheDir;
use itertools::Itertools;
use yaml_rust::YamlEmitter;

pub use ambient_ecs::generated::components::core::ecs::{children, parent};

use crate::{asset_cache, name};

pub fn despawn_recursive(world: &mut World, entity: EntityId) {
    if let Ok(children) = world.set(entity, children(), vec![]) {
        for c in children {
            despawn_recursive(world, c);
        }
    }
    world.despawn(entity);
}
pub fn despawn_children_recursive(world: &mut World, entity: EntityId) {
    if let Ok(children) = world.set(entity, children(), vec![]) {
        for c in children {
            despawn_recursive(world, c);
        }
    }
}
pub fn add_child(world: &mut World, id: EntityId, child_id: EntityId) -> Result<(), ECSError> {
    if world.has_component(id, children()) {
        world.get_mut(id, children())?.push(child_id);
    } else {
        world.add_component(id, children(), vec![child_id])?;
    }
    Ok(())
}

pub fn find_child<F: Fn(&World, EntityId) -> bool>(
    world: &World,
    entity: EntityId,
    query: &F,
) -> Option<EntityId> {
    if let Ok(children) = world.get_ref(entity, children()) {
        for child in children {
            if query(world, *child) {
                return Some(*child);
            }
            if let Some(hit) = find_child(world, *child, query) {
                return Some(hit);
            }
        }
    }
    None
}
pub fn apply_recursive<F: Fn(&mut World, EntityId)>(world: &mut World, entity: EntityId, func: &F) {
    func(world, entity);
    if let Ok(children) = world.get_ref(entity, children()).map(|x| x.clone()) {
        for child in children {
            apply_recursive(world, child, func);
        }
    }
}
pub fn set_component_recursive<T: ComponentValue>(
    world: &mut World,
    entity: EntityId,
    component: Component<T>,
    value: T,
) {
    apply_recursive(world, entity, &|world, entity| {
        if world.has_component(entity, component) {
            world.set(entity, component, value.clone()).unwrap();
        }
    })
}
pub fn find_child_with_name_ending(
    world: &World,
    entity: EntityId,
    name_ending: &str,
) -> Option<EntityId> {
    find_child(world, entity, &|world, id| {
        if let Ok(name) = world.get_ref(id, name()) {
            name.ends_with(name_ending)
        } else {
            false
        }
    })
}

#[cfg(not(target_os = "unknown"))]
fn dump_world_hierarchy_to_tmp_file(world: &World) {
    use std::{fs::File, path::PathBuf};

    use ambient_std::asset_cache::SyncAssetKeyExt;

    let cache_dir = world
        .resource_opt(asset_cache())
        .map(|a| AssetsCacheDir.get(a))
        .unwrap_or(PathBuf::from("tmp"));
    std::fs::create_dir_all(&cache_dir).ok();
    let path = cache_dir.join("hierarchy.yml");
    let mut f = File::create(&path).expect("Unable to create file");
    dump_world_hierarchy(world, &mut f);

    tracing::info!("Wrote hierarchy to {path:?}");
}

pub fn dump_world_hierarchy_to_clipboard(world: &World) {
    let mut s = Vec::new();
    dump_world_hierarchy(world, &mut s);

    let s = String::from_utf8_lossy(&s);

    ambient_sys::clipboard::set_background(s, |res| match res {
        Ok(()) => tracing::info!("Dumped world hierarchy to clipboard"),
        Err(err) => tracing::error!("Failed to dump world hierarchy to clipboard: {err:?}"),
    });
}

pub fn dump_world_hierarchy_to_user(world: &World) {
    #[cfg(target_os = "unknown")]
    dump_world_hierarchy_to_clipboard(world);
    #[cfg(not(target_os = "unknown"))]
    dump_world_hierarchy_to_tmp_file(world);
}

pub fn dump_world_hierarchy(world: &World, f: &mut dyn std::io::Write) {
    use yaml_rust::yaml::Yaml;
    let mut visited = HashSet::new();
    let mut roots = query(())
        .excl(parent())
        .iter(world, None)
        .map(|(id, _)| yaml_rust::yaml::Yaml::Hash(dump_entity_hierarchy(world, id, &mut visited)))
        .collect_vec();

    let orphaned = query(())
        .iter(world, None)
        .filter(|(id, _)| !visited.contains(id))
        .map(|(id, _)| {
            let (_, entity_yml) = world.dump_entity_to_yml(id).unwrap();
            Yaml::Hash(entity_yml)
        })
        .collect_vec();

    if !orphaned.is_empty() {
        let mut res = yaml_rust::yaml::Hash::new();
        res.insert(Yaml::String("ORPHANED".to_string()), Yaml::Array(orphaned));
        roots.insert(0, Yaml::Hash(res));
    }

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.multiline_strings(false);
    emitter.dump(&yaml_rust::yaml::Yaml::Array(roots)).unwrap();
    write!(f, "{out_str}").unwrap();
}
fn dump_entity_hierarchy(
    world: &World,
    entity: EntityId,
    visited: &mut HashSet<EntityId>,
) -> yaml_rust::yaml::Hash {
    use yaml_rust::yaml::Yaml;
    visited.insert(entity);
    let mut res = yaml_rust::yaml::Hash::new();
    if let Some((id, entity_yml)) = world.dump_entity_to_yml(entity) {
        let element = if let Some(Yaml::String(element)) =
            entity_yml.get(&Yaml::String("element".to_string()))
        {
            Some(element.clone())
        } else {
            None
        };
        let name =
            if let Some(Yaml::String(name)) = entity_yml.get(&Yaml::String("name".to_string())) {
                Some(name.clone())
            } else {
                None
            };
        let key = vec![Some(id), element, name]
            .into_iter()
            .flatten()
            .join(" • ");
        res.insert(Yaml::String(key), Yaml::Hash(entity_yml));
        res.insert(
            Yaml::String("children".to_string()),
            Yaml::Array(if let Ok(children) = world.get_ref(entity, children()) {
                children
                    .iter()
                    .map(|c| yaml_rust::yaml::Yaml::Hash(dump_entity_hierarchy(world, *c, visited)))
                    .collect_vec()
            } else {
                Vec::new()
            }),
        );
    } else {
        res.insert(
            yaml_rust::yaml::Yaml::String("error".to_string()),
            yaml_rust::yaml::Yaml::String("no_such_entity".to_string()),
        );
    }
    res
}
