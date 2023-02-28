use std::{collections::HashMap, sync::Arc};

use ambient_core::{
    asset_cache,
    async_ecs::{async_run, AsyncRun},
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    hierarchy::{children, despawn_recursive},
    main_scene, runtime,
    transform::{get_world_position, inv_local_to_world, local_to_world, mesh_to_world},
};
use ambient_ecs::{
    components, query, ComponentDesc, Debuggable, Description, EntityData, EntityId, MaybeResource, Name, Networked, Store, SystemGroup,
    World,
};
use ambient_gpu::mesh_buffer::GpuMeshFromUrl;
use ambient_renderer::{
    color, gpu_primitives,
    materials::{
        flat_material::{get_flat_shader, FlatMaterialKey},
        pbr_material::{get_pbr_shader, PbrMaterialFromUrl},
    },
    primitives, RenderPrimitive, StandardShaderKey,
};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, AssetUrl, ModelAssetType, TypedAssetUrl},
    cb,
    download_asset::{AssetError, BytesFromUrl, JsonFromUrl},
    log_result,
    math::Line,
};
use async_trait::async_trait;
use futures::StreamExt;
use glam::{vec4, Vec3};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
mod model;

use ambient_meshes::CubeMeshKey;
pub use model::*;
use tokio::sync::Semaphore;

use self::loading_material::{LoadingMaterialKey, LoadingShaderKey};
use anyhow::Context;

pub mod loading_material;

components!("model", {
    @[Networked, Store]
    animation_binder: HashMap<String, EntityId>,
    @[Debuggable, Networked, Store]
    animation_bind_id: String,

    model: Arc<Model>,
    @[Debuggable, Networked, Store, Name["Model from URL"], Description["Load a model from the given URL or relative path."]]
    model_from_url: String,

    @[Networked, Store]
    pbr_renderer_primitives_from_url: Vec<PbrRenderPrimitiveFromUrl>,
    @[Debuggable, Networked, Store, MaybeResource, Name["Model animatable"], Description["Controls whether this model can be animated."]]
    model_animatable: bool,
    @[Networked, Store, MaybeResource]
    model_skins: Vec<ModelSkin>,
    @[Networked, Store]
    model_skin_ix: usize,

    @[Debuggable, Networked, Store, Name["Model loaded"], Description["If attached, this entity has a model attached to it."]]
    model_loaded: (),
    @[Debuggable, Networked, Store]
    is_model_node: (),
});

#[tracing::instrument(skip(assets, async_run))]
async fn internal_spawn_models_from_defs(
    assets: &AssetCache,
    async_run: AsyncRun,
    entities_with_models: HashMap<String, Vec<EntityId>>,
) -> anyhow::Result<()> {
    // Meanwhile, spawn a spinning cube onto the entity.
    let cube = CubeMeshKey.get(assets);

    let mat = LoadingMaterialKey { speed: 2.0, scale: 6.0 }.get(assets);

    let cube = EntityData::new()
        .set(
            primitives(),
            vec![RenderPrimitive {
                shader: cb(move |assets, config| {
                    StandardShaderKey { material_shader: LoadingShaderKey.get(assets), lit: false, shadow_cascades: config.shadow_cascades }
                        .get(assets)
                }),
                material: mat,
                mesh: cube,
                lod: 0,
            }],
        )
        .set_default(gpu_primitives())
        .set(color(), vec4(0.0, 0.5, 1.0, 1.0))
        .set(main_scene(), ())
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set_default(local_to_world())
        .set_default(inv_local_to_world());

    let mut ids = entities_with_models.values().flatten().copied().collect_vec();

    let cube_fail = Arc::new(cube.clone().set(color(), vec4(1.0, 0.0, 0.0, 1.0)));

    async_run.run(move |world| {
        ids.retain(|id| world.exists(*id));
        for id in ids {
            remove_model(world, id);
            tracing::debug!("Spawning cube model for {id}");
            log_result!(world.add_components(id, cube.clone()))
        }
    });

    let iter = entities_with_models.into_iter().map(|(url, ids)| async move {
        tracing::debug!("Loading model: {url:#?}");
        let mut url = match TypedAssetUrl::parse(url).context("Failed to parse url") {
            Ok(url) => url,
            Err(e) => return (ids, Err(e)),
        };
        if !url.0.path().contains("/models/") {
            url = match url.0.as_directory().join("models/main.json").context("Failed to join url") {
                Ok(url) => url.into(),
                Err(e) => return (ids, Err(e)),
            };
        };
        match ModelFromUrl(url).get(assets).await.context("Failed to load model") {
            Ok(v) => (ids, Ok(v)),
            Err(e) => (ids, Err(e)),
        }
    });

    let mut iter = futures::stream::iter(iter).buffer_unordered(4);
    while let Some((mut ids, model)) = iter.next().await {
        let cube_fail = cube_fail.clone();
        async_run.run(move |world| {
            // Remove the models which still exist
            ids.retain(|id| world.exists(*id));
            for id in &ids {
                remove_model(world, *id);
            }

            let len = ids.len();

            match model {
                Ok(model) => {
                    tracing::info!("Spawning model: {:?} for {ids:?}", model.name());
                    model.batch_spawn(
                        world,
                        &ModelSpawnOpts {
                            root: ModelSpawnRoot::AttachTo(ids),
                            // We need to keep the model alive on the entity here, or otherwise it'll unload from the asset store
                            root_components: EntityData::new().set(self::model(), model.clone()),
                            ..Default::default()
                        },
                        len,
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to load model: {e:?}");
                    for id in ids {
                        remove_model(world, id);
                        tracing::debug!("Spawning cube model for {id}");
                        log_result!(world.add_components(id, (*cube_fail).clone()))
                    }
                }
            }
        })
    }
    Ok(())
}

pub fn model_systems() -> SystemGroup {
    SystemGroup::new(
        "model_systems",
        vec![
            query((children(),)).incl(model_from_url()).despawned().to_system(|q, world, qs, _| {
                for (_, (children,)) in q.collect_cloned(world, qs) {
                    for c in children {
                        if world.has_component(c, is_model_node()) {
                            despawn_recursive(world, c);
                        }
                    }
                }
            }),
            query(()).incl(model_from_url()).despawned().to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    remove_model(world, id);
                }
            }),
            query((model_from_url().changed(),)).to_system(|q, world, qs, _| {
                let mut new_models = HashMap::<String, Vec<EntityId>>::new();
                for (id, (model_from_url,)) in q.iter(world, qs) {
                    let entry = new_models.entry(model_from_url.clone()).or_default();
                    entry.push(id);
                }
                if new_models.is_empty() {
                    return;
                }
                let assets = world.resource(asset_cache()).clone();
                let runtime = world.resource(runtime()).clone();
                let async_run = world.resource(async_run()).clone();

                runtime.spawn(async move { internal_spawn_models_from_defs(&assets, async_run, new_models).await });
            }),
        ],
    )
}
fn remove_model(world: &mut World, entity: EntityId) {
    if let Ok(mut childs) = world.get_ref(entity, children()).map(|cs| cs.clone()) {
        childs.retain(|c| {
            if world.has_component(*c, is_model_node()) {
                despawn_recursive(world, *c);
                false
            } else {
                true
            }
        });
        world.set(entity, children(), childs).ok();
    }
    let mut components: Vec<ComponentDesc> = vec![
        primitives().desc(),
        gpu_primitives().desc(),
        animation_binder().desc(),
        local_bounding_aabb().desc(),
        world_bounding_aabb().desc(),
        world_bounding_sphere().desc(),
        model_loaded().desc(),
    ];
    components.retain(|&comp| world.has_component_ref(entity, comp));
    world.remove_components(entity, components).ok();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFromUrl(pub TypedAssetUrl<ModelAssetType>);
impl ModelFromUrl {
    pub fn new(url: impl AsRef<str>) -> anyhow::Result<Self> {
        Ok(Self(TypedAssetUrl::parse(url)?))
    }
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<Model>, AssetError>> for ModelFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<Model>, AssetError> {
        let url = self.0.clone().abs().context(format!("ModelFromUrl got relative url: {}", self.0))?;
        let data = BytesFromUrl::new(url.clone(), true).get(&assets).await?;
        let semaphore = ModelLoadSemaphore.get(&assets);
        let _permit = semaphore.acquire().await;
        let mut model = ambient_sys::task::block_in_place(|| Model::from_slice(&data))?;
        model.load(&assets, &url).await?;
        Ok(Arc::new(model))
    }
}

/// Limit the number of concurent model loads to 10
#[derive(Debug)]
struct ModelLoadSemaphore;
impl SyncAssetKey<Arc<Semaphore>> for ModelLoadSemaphore {
    fn load(&self, _assets: AssetCache) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(10))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PbrRenderPrimitiveFromUrl {
    pub mesh: AssetUrl,
    pub material: Option<AssetUrl>,
    pub lod: usize,
}
impl PbrRenderPrimitiveFromUrl {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<PbrRenderPrimitiveFromResolvedUrl> {
        Ok(PbrRenderPrimitiveFromResolvedUrl {
            mesh: self.mesh.resolve(base_url)?,
            material: if let Some(x) = &self.material { Some(x.resolve(base_url)?) } else { None },
            lod: self.lod,
        })
    }
}
#[derive(Debug, Clone)]
pub struct PbrRenderPrimitiveFromResolvedUrl {
    pub mesh: AbsAssetUrl,
    pub material: Option<AbsAssetUrl>,
    pub lod: usize,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<RenderPrimitive>, AssetError>> for PbrRenderPrimitiveFromResolvedUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<RenderPrimitive>, AssetError> {
        let mesh = GpuMeshFromUrl { url: self.mesh, cache_on_disk: true }.get(&assets).await?;
        if let Some(mat_url) = self.material {
            let mat_def = JsonFromUrl::<PbrMaterialFromUrl>::new(mat_url.clone(), true).get(&assets).await?;
            let mat = mat_def.resolve(&mat_url)?.get(&assets).await?;
            Ok(Arc::new(RenderPrimitive { material: mat.into(), shader: cb(get_pbr_shader), mesh, lod: self.lod }))
        } else {
            Ok(Arc::new(RenderPrimitive {
                material: FlatMaterialKey::white().get(&assets),
                shader: cb(get_flat_shader),
                mesh,
                lod: self.lod,
            }))
        }
    }
}

pub fn bones_to_lines(world: &World, id: EntityId) -> Vec<Line> {
    fn inner(world: &World, id: EntityId, pos: Vec3, lines: &mut Vec<Line>) {
        let children = world.get_ref(id, children());
        for &c in children.into_iter().flatten() {
            let child_pos = get_world_position(world, c);
            if let Ok(child_pos) = child_pos {
                lines.push(Line(pos, child_pos));
                inner(world, c, child_pos, lines);
            }
        }
    }

    let pos = get_world_position(world, id);
    let mut lines = Vec::new();
    if let Ok(pos) = pos {
        inner(world, id, pos, &mut lines);
    }
    lines
}
