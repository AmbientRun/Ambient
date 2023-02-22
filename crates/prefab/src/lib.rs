use std::{collections::HashMap, sync::Arc};

use ambient_core::{asset_cache, async_ecs::async_run, hierarchy::children, runtime};
use ambient_decals::decal;
use ambient_ecs::{
    components, query, query_mut, Debuggable, Description, DeserWorldWithWarnings, EntityId, Name, Networked, Store, SystemGroup, World,
};
use ambient_model::model_from_url;
use ambient_physics::collider::collider;
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt},
    asset_url::AssetUrl,
    download_asset::{AssetError, BytesFromUrl},
    unwrap_log_err,
};
use anyhow::Context;
use async_trait::async_trait;

components!("prefab", {
    @[
        Debuggable, Networked, Store,
        Name["Prefab from URL"],
        Description["Load and attach a prefab from a URL or relative path.\nWhen loaded, the components from this prefab will add to or replace the existing components for the entity."]
    ]
    prefab_from_url: String,
    @[
        Debuggable,
        Name["Spawned"],
        Description["If attached, this entity was built from a prefab that has finished spawning."]
    ]
    spawned: (),
});

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "prefab",
        vec![query(prefab_from_url()).spawned().to_system(|q, world, qs, _| {
            let mut to_load = HashMap::<String, Vec<EntityId>>::new();
            for (id, url) in q.collect_cloned(world, qs) {
                let url = if url.ends_with("/prefabs/main.json") { url } else { format!("{url}/prefabs/main.json") };
                to_load.entry(url).or_default().push(id);
            }
            for (url, ids) in to_load {
                let assets = world.resource(asset_cache()).clone();
                let url = unwrap_log_err!(AssetUrl::parse(url));
                let url = PrefabFromUrl(url);
                let runtime = world.resource(runtime()).clone();
                let async_run = world.resource(async_run()).clone();
                runtime.spawn(async move {
                    let obj = unwrap_log_err!(url.get(&assets).await);
                    let base_ent_id = obj.resource(children())[0];
                    // TODO: This only handles prefabs with a single entity
                    let entity = obj.clone_entity(base_ent_id).unwrap();
                    async_run.run(move |world| {
                        for id in ids {
                            world.add_components(id, entity.clone()).unwrap();
                            world.add_component(id, spawned(), ()).unwrap();
                        }
                    });
                });
            }
        })],
    )
}

#[derive(Debug, Clone)]
pub struct PrefabFromUrl(pub AssetUrl);
#[async_trait]
impl AsyncAssetKey<Result<Arc<World>, AssetError>> for PrefabFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<World>, AssetError> {
        let obj_url = self.0.abs().context(format!("PrefabFromUrl got relative url: {}", self.0))?;
        let data = BytesFromUrl::new(obj_url.clone(), true).get(&assets).await?;
        let DeserWorldWithWarnings { mut world, warnings } = tokio::task::block_in_place(|| serde_json::from_slice(&data))
            .with_context(|| format!("Failed to deserialize object2 from url {obj_url}"))?;
        warnings.log_warnings();
        for (_id, (url,), _) in query_mut((model_from_url(),), ()).iter(&mut world, None) {
            *url = AssetUrl::parse(&url).context("Invalid model url")?.resolve(&obj_url).context("Failed to resolve model url")?.into();
        }
        for (_id, (def,), _) in query_mut((collider(),), ()).iter(&mut world, None) {
            def.resolve(&obj_url).context("Failed to resolve collider")?;
        }
        for (_id, (def,), _) in query_mut((decal(),), ()).iter(&mut world, None) {
            *def = def.resolve(&obj_url).context("Failed to resolve decal")?.into();
        }
        Ok(Arc::new(world))
    }
}
