use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use ambient_core::{
    asset_cache,
    async_ecs::{async_run, AsyncRun},
    runtime,
    transform::{local_to_world, translation},
};
use ambient_ecs::{components, query, Entity, EntityId, FnSystem, SystemGroup};
use ambient_gpu::gpu::GpuKey;
use ambient_model::{Model, ModelFromUrl, ModelSpawnOpts, ModelSpawnRoot};
use ambient_renderer::color;
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    log_result,
};
use ambient_terrain::{terrain_cell_version, terrain_state, TerrainState};
use glam::{vec4, EulerRot, Mat4, Quat, UVec3, Vec3, Vec4};
use itertools::Itertools;
use rand::prelude::SliceRandom;

mod compute;
mod config;
use ambient_network::ServerWorldExt;
pub use compute::*;
pub use config::*;
use tokio::sync::Semaphore;

components!("game_objects", {
    natural_entities: HashMap<String, Vec<EntityId>>,
    // Keep a ref to the model on each natural, so that the model doesn't get unloaded while there are any naturals of that type
    natural_model: Arc<Model>,
    natural_layers: Vec<NaturalLayer>,
    natural_layers_in_progress: usize,
    terrain_cell_nature_version: i32,
    terrain_cell_nature_conf_hash: u64,
});

pub fn init_world_resources() -> Entity {
    Entity::new()
}

const NATURALS_MAX_ENTITIES: u32 = 1_000_000;
const WORKGROUP_SIZE: u32 = 32;
const MAX_CONCURRENT: usize = 3;
pub(crate) static OLD_CONTENT_SERVER_URL: &str = "https://fra1.digitaloceanspaces.com/dims-content/";

#[derive(Debug)]
struct NaturalsSemaphore;
impl SyncAssetKey<Arc<Semaphore>> for NaturalsSemaphore {
    fn load(&self, _assets: AssetCache) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(MAX_CONCURRENT))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NaturalEntity {
    position: Vec3,
    scale: f32,
    rotation: Vec3,
    element: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NaturalElementWGSL {
    soil_depth: NaturalCurveWGSL,
    elevation: NaturalCurveWGSL,
    water_depth: NaturalCurveWGSL,
    steepness: NaturalCurveWGSL,
    cluster_noise: NaturalCurveWGSL,

    scale_min: f32,
    scale_max: f32,
    scale_power: f32,
    rotation_x: f32,
    rotation_y: f32,
    rotation_z: f32,
    rotation_z_jitter: f32,
    rotation_xy_jitter: f32,
    rotation_straightness: f32,
    position_normal_offset: f32,
    position_z_offset: f32,
    normal_miplevel: f32,
    cluster_noise_scale: f32,

    _padding: UVec3,
}
impl From<NaturalElement> for NaturalElementWGSL {
    fn from(element: NaturalElement) -> Self {
        Self {
            scale_min: element.scale_min,
            scale_max: element.scale_max,
            scale_power: element.scale_power,
            rotation_x: element.rotation_x,
            rotation_y: element.rotation_y,
            rotation_z: element.rotation_z,
            rotation_z_jitter: element.rotation_z_jitter,
            rotation_xy_jitter: element.rotation_xy_jitter,
            rotation_straightness: element.rotation_straightness,
            position_normal_offset: element.position_normal_offset,
            position_z_offset: element.position_z_offset,
            normal_miplevel: element.normal_miplevel,
            cluster_noise_scale: element.cluster_noise_scale,

            soil_depth: element.soil_depth.into(),
            elevation: element.elevation.into(),
            water_depth: element.water_depth.into(),
            steepness: element.steepness.into(),
            cluster_noise: element.cluster_noise.into(),

            _padding: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NaturalCurveWGSL {
    params: Vec4,
    kind: u32,
    _padding: UVec3,
}
impl From<NaturalCurve> for NaturalCurveWGSL {
    fn from(curve: NaturalCurve) -> Self {
        match curve {
            NaturalCurve::Constant { value } => Self { kind: 0, params: vec4(value, 0., 0., 0.), _padding: Default::default() },
            NaturalCurve::Interpolate { x0, x1, y0, y1 } => Self { kind: 1, params: vec4(x0, x1, y0, y1), _padding: Default::default() },
            NaturalCurve::InterpolateClamped { x0, x1, y0, y1 } => {
                Self { kind: 2, params: vec4(x0, x1, y0, y1), _padding: Default::default() }
            }
            NaturalCurve::SmoothStep { x0, x1, y0, y1 } => Self { kind: 3, params: vec4(x0, x1, y0, y1), _padding: Default::default() },
            NaturalCurve::BellCurve { center, width, y0, y1 } => {
                Self { kind: 4, params: vec4(center, width, y0, y1), _padding: Default::default() }
            }
        }
    }
}

async fn update_natural_layer(
    assets: AssetCache,
    async_run: AsyncRun,
    terrain_state: TerrainState,
    terrain_cell_id: EntityId,
    terrain_cell_position: Vec3,
    layer: NaturalLayer,
) {
    // Flatten elements so that there's one element per model
    let elements: Vec<(NaturalElement, BoxModelKey)> = layer
        .elements
        .into_iter()
        .filter(|el| el.enabled)
        .flat_map(|element| {
            element
                .models
                .iter()
                .flat_map(|model| {
                    model
                        .0
                        .iter()
                        .filter_map(|url| {
                            let model = Box::new(ModelFromUrl(url.join("../models/main.json").ok()?.into())) as BoxModelKey;
                            Some((element.clone(), model))
                        })
                        .collect_vec()
                })
                .collect_vec()
        })
        .collect_vec();
    if elements.is_empty() {
        return;
    }
    let naturals = NaturalsPipelineKey.get(&assets).await;
    let semaphore = NaturalsSemaphore.get(&assets);
    let permit = semaphore.acquire().await;

    let gpu = GpuKey.get(&assets);
    let entities = naturals.run(&gpu, &elements, layer.grid_size, &terrain_state).await;

    drop(permit); // This permit is only to make sure we don't use too much GPU memory

    let mut entities_by_element = (0..elements.len()).map(|_| Vec::new()).collect_vec();

    for entity in entities.into_iter() {
        entities_by_element[entity.element as usize].push(entity);
    }

    for ((element, model_def), entities) in elements.into_iter().zip(entities_by_element.into_iter()) {
        let assets = assets.clone();
        let gpu = GpuKey.get(&assets);
        let async_run = async_run.clone();
        tokio::spawn(async move {
            let model_key = model_def.key();
            let model = if !entities.is_empty() { model_def.get(&assets).await.map(Some) } else { Ok(None) };
            match model {
                Ok(model) => {
                    async_run.run(move |world| {
                        let mut existing = {
                            let ents = world.get_mut(terrain_cell_id, natural_entities()).unwrap();
                            ents.entry(model_key.clone()).or_insert(Vec::new()).clone()
                        };
                        if let Some(model) = model {
                            let missing = entities.len() as i32 - existing.len() as i32;
                            if missing > 0 {
                                let ids = model.batch_spawn(
                                    &gpu,
                                    world,
                                    &ModelSpawnOpts {
                                        root: ModelSpawnRoot::Spawn,
                                        root_components: Entity::new()
                                            .with(natural_model(), model.clone())
                                            .with(color(), element.color.into())
                                            .with_default(local_to_world()),
                                        animatable: Some(false),
                                        ..Default::default()
                                    },
                                    missing as usize,
                                );
                                existing.extend(ids.into_iter());
                            }
                        }
                        while existing.len() > entities.len() {
                            let id = existing.pop().unwrap();
                            world.despawn(id);
                        }
                        for (id, ent) in existing.iter().zip(entities.iter()) {
                            world
                                .set_if_changed(
                                    *id,
                                    local_to_world(),
                                    Mat4::from_scale_rotation_translation(
                                        Vec3::ONE * ent.scale,
                                        Quat::from_euler(EulerRot::XYZ, ent.rotation.x, ent.rotation.y, ent.rotation.z),
                                        terrain_cell_position + ent.position,
                                    ),
                                )
                                .unwrap();
                        }
                        let ents = world.get_mut(terrain_cell_id, natural_entities()).unwrap();
                        ents.insert(model_key, existing);
                        *world.get_mut(terrain_cell_id, natural_layers_in_progress()).unwrap() -= 1;
                    });
                }
                Err(err) => {
                    tracing::warn!("Failed to load asset {}: {:#}", model_key, err);
                    async_run.run(move |world| {
                        *world.get_mut(terrain_cell_id, natural_layers_in_progress()).unwrap() -= 1;
                    });
                }
            }
        });
    }
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "dims/naturals/client_systems",
        vec![
            query(()).incl(terrain_state()).excl(natural_entities()).to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    log_result!(world.add_components(
                        id,
                        Entity::new()
                            .with(terrain_cell_nature_conf_hash(), 0u64)
                            .with(terrain_cell_nature_version(), -1)
                            .with_default(natural_layers_in_progress())
                            .with_default(natural_entities()),
                    ));
                }
            }),
            Box::new(FnSystem::new(|world, _| {
                let layers = world
                    .persisted_resource(natural_layers())
                    .cloned()
                    .unwrap_or_else(|| get_default_natural_layers(NaturalsPreset::Mountains));
                let layers = layers
                    .into_iter()
                    .filter_map(|layer| {
                        let res = NaturalLayer { elements: layer.elements.into_iter().filter(|e| e.enabled).collect_vec(), ..layer };
                        if !res.elements.is_empty() {
                            Some(res)
                        } else {
                            None
                        }
                    })
                    .collect_vec();
                let layers_hash = calculate_hash(&format!("{layers:?}"));

                let updatable = query((
                    terrain_cell_nature_conf_hash(),
                    terrain_cell_version(),
                    terrain_cell_nature_version(),
                    natural_layers_in_progress(),
                ))
                .iter(world, None)
                .filter_map(
                    |(id, (hash, version, nature_version, in_progress))| {
                        if *in_progress == 0 && (*version != *nature_version || *hash != layers_hash) {
                            Some(id)
                        } else {
                            None
                        }
                    },
                )
                .collect_vec();

                let terrain_cell_id = match updatable.choose(&mut rand::thread_rng()) {
                    Some(id) => *id,
                    None => return,
                };

                world.set(terrain_cell_id, terrain_cell_nature_conf_hash(), layers_hash).unwrap();
                let version = world.get(terrain_cell_id, terrain_cell_version()).unwrap();
                world.set(terrain_cell_id, terrain_cell_nature_version(), version).unwrap();

                let valid_models = layers
                    .iter()
                    .flat_map(|layer| layer.elements.iter().flat_map(|element| element.models.iter().flat_map(|models| models.0.clone())))
                    .map(|x| x.to_string())
                    .collect::<HashSet<String>>();

                let state = world.get_ref(terrain_cell_id, terrain_state()).unwrap().clone();
                let cell_pos = *world.get_ref(terrain_cell_id, translation()).unwrap();

                for layer in layers.into_iter() {
                    if layer.grid_size <= 0.05 || layer.grid_size >= 1000. {
                        continue;
                    }
                    *world.get_mut(terrain_cell_id, natural_layers_in_progress()).unwrap() +=
                        layer.elements.iter().flat_map(|el| el.models.iter().map(|model| model.0.len())).sum::<usize>();
                    let async_run = world.resource(async_run()).clone();
                    let assets = world.resource(asset_cache()).clone();
                    let state = state.clone();
                    world.resource(runtime()).spawn(async move {
                        update_natural_layer(assets, async_run, state, terrain_cell_id, cell_pos, layer).await;
                    });
                }

                // Clean out invalid models
                let to_remove = {
                    let ents = world.get_mut(terrain_cell_id, natural_entities()).unwrap();
                    let models_to_remove =
                        ents.keys().filter_map(|key| if !valid_models.contains(key) { Some(key.to_string()) } else { None }).collect_vec();
                    let entities = models_to_remove.iter().flat_map(|key| ents.get(key).unwrap().clone()).collect_vec();
                    for key in models_to_remove.iter() {
                        ents.remove(key);
                    }
                    entities
                };
                for id in to_remove {
                    world.despawn(id);
                }
            })),
        ],
    )
}

fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {
    use std::hash::Hasher;
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
