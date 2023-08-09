use std::{
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use ambient_ecs::{components, Resource, World};
use ambient_native_std::asset_url::AbsAssetUrl;
use ambient_project_semantic::{ItemId, Scope, Semantic};

components!("semantic", {
    @[Resource]
    semantic: Arc<Mutex<Semantic>>,
});

pub fn world_semantic(world: &World) -> Arc<Mutex<Semantic>> {
    world.resource(semantic()).clone()
}

/// Returns the path for the given file in the given ember, or returns a global path
/// if that ember doesn't have an associated URL.
///
/// Note that `path` is relative to the root of the ember's build directory, so an
/// asset will require `assets/` prefixed to the path.
pub fn file_path(
    semantic: &Semantic,
    ember_id: ItemId<Scope>,
    path: &Path,
) -> anyhow::Result<AbsAssetUrl> {
    let item = semantic.items.get(ember_id)?;
    if let Some(url) = item.url.as_ref() {
        Ok(AbsAssetUrl::from_str(&format!("{url}/{}", path.display()))?)
    } else {
        Ok(AbsAssetUrl::from_asset_key(path.to_string_lossy())?)
    }
}
