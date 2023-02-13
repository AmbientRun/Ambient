use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use kiwi_core::{asset_cache, async_ecs::async_run, hierarchy::children, runtime};
use kiwi_decals::decal;
use kiwi_ecs::{
    components, query, query_mut, Debuggable, Description, DeserWorldWithWarnings, EntityId, Name, Networked, Store, SystemGroup, World,
};
use kiwi_model::{model_from_url, ModelFromUrl};
use kiwi_physics::collider::collider;
use kiwi_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, AssetUrl, ServerBaseUrlKey},
    download_asset::{AssetError, BytesFromUrl},
    unwrap_log_err,
};

components!("object", {
    @[
        Debuggable, Networked, Store,
        Name["Object from URL"],
        Description["Load and attach an object from a URL or relative path.\nWhen loaded, the components from this object will add to or replace the existing components for the object."]
    ]
    object_from_url: String,
});

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "object",
        vec![query(object_from_url()).spawned().to_system(|q, world, qs, _| {
            let mut to_load = HashMap::<String, Vec<EntityId>>::new();
            for (id, url) in q.collect_cloned(world, qs) {
                let url = if url.ends_with("/objects/main.json") { url } else { format!("{url}/objects/main.json") };
                to_load.entry(url).or_default().push(id);
            }
            for (url, ids) in to_load {
                let assets = world.resource(asset_cache()).clone();
                let url = unwrap_log_err!(AssetUrl::parse(url));
                let url = ObjectFromUrl(url);
                let runtime = world.resource(runtime()).clone();
                let async_run = world.resource(async_run()).clone();
                runtime.spawn(async move {
                    let obj = unwrap_log_err!(url.get(&assets).await);
                    let base_ent_id = obj.resource(children())[0];
                    // TODO: This only handles objects with a single entity
                    let entity = obj.clone_entity(base_ent_id).unwrap();
                    async_run.run(move |world| {
                        for id in ids {
                            world.add_components(id, entity.clone()).unwrap();
                        }
                    });
                });
            }
        })],
    )
}

#[derive(Debug, Clone)]
pub struct ObjectFromUrl(pub AssetUrl);
#[async_trait]
impl AsyncAssetKey<Result<Arc<World>, AssetError>> for ObjectFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<World>, AssetError> {
        let obj_url = self.0.resolve(&ServerBaseUrlKey.get(&assets)).context("Failed to resolve url")?;
        let data = BytesFromUrl::new(obj_url.clone(), true).get(&assets).await?;
        let DeserWorldWithWarnings { mut world, warnings } = tokio::task::block_in_place(|| serde_json::from_slice(&data))
            .with_context(|| format!("Failed to deserialize object2 from url {}", obj_url))?;
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
