use std::{collections::HashSet, sync::Arc};

use ambient_animation::{
    animation_controller, AnimationActionTime, AnimationClip, AnimationClipFromUrl,
};
use ambient_core::{asset_cache, transform::translation};
use ambient_ecs::{query as ecs_query, with_component_registry, EntityId, World};
use ambient_model::ModelFromUrl;
use ambient_network::ServerWorldExt;
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt},
    asset_url::{AnimationAssetType, TypedAssetUrl},
};
use anyhow::Context;

use super::{
    super::{
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
    component::convert_components_to_entity_data,
};

pub fn spawn(
    world: &mut World,
    spawned_entities: &mut HashSet<EntityId>,
    data: wit::entity::EntityData,
) -> anyhow::Result<wit::types::EntityId> {
    let id = convert_components_to_entity_data(data).spawn(world);
    spawned_entities.insert(id);
    Ok(id.into_bindgen())
}

pub fn despawn(
    world: &mut World,
    spawned_entities: &mut HashSet<EntityId>,
    id: wit::types::EntityId,
) -> anyhow::Result<bool> {
    let id = id.from_bindgen();
    spawned_entities.remove(&id);
    Ok(world.despawn(id).is_some())
}

pub fn set_animation_controller(
    world: &mut World,
    entity: wit::types::EntityId,
    controller: wit::entity::AnimationController,
) -> anyhow::Result<()> {
    Ok(world.add_component(
        entity.from_bindgen(),
        animation_controller(),
        controller.from_bindgen(),
    )?)
}


pub fn set_animation_blend(
    world: &mut World,
    entity: wit::types::EntityId,
    weights: &[f32],
    times: &[f32],
    absolute_time: bool,
) -> anyhow::Result<()> {
    let controller = world.get_mut(entity.from_bindgen(), animation_controller())?;
    for (action, weight) in controller.actions.iter_mut().zip(weights.iter()) {
        action.weight = *weight;
    }

    if absolute_time {
        for (action, time) in controller.actions.iter_mut().zip(times.iter()) {
            action.time = AnimationActionTime::Absolute { time: *time };
        }
    } else {
        for (action, time) in controller.actions.iter_mut().zip(times.iter()) {
            action.time = AnimationActionTime::Percentage { percentage: *time }
        }
    }
    Ok(())
}

fn peek_loaded_clip(
    assets: &AssetCache,
    clip_url: &str,
) -> anyhow::Result<Option<Arc<AnimationClip>>> {
    let asset_url: TypedAssetUrl<AnimationAssetType> =
        TypedAssetUrl::parse(clip_url).context("Invalid clip url")?;
    let clip_asset_url: TypedAssetUrl<AnimationAssetType> = asset_url
        .abs()
        .context(format!("Expected absolute url, got: {}", clip_url))?
        .into();

    if let Some(asset) = ModelFromUrl(
        clip_asset_url
            .model_crate()
            .context("Invalid clip url")?
            .model(),
    )
    .peek(&assets)
    {
        let _model = asset.context("No such model")?;
    } else {
        return Ok(None);
    }

    if let Some(clip) = AnimationClipFromUrl::new(asset_url.unwrap_abs(), true).peek(assets) {
        Ok(Some(clip.context("No such clip")?))
    } else {
        Ok(None)
    }
}

pub fn has_animation_clip(world: &mut World, clip_url: &str) -> anyhow::Result<bool> {
    let assets = world.resource(asset_cache());
    if let Ok(clip) = peek_loaded_clip(assets, clip_url) {
        return Ok(clip.is_some());
    }

    Ok(true)
}

pub fn get_animation_clips(
    world: &mut World,
    clip_urls: &[String],
) -> anyhow::Result<Vec<wit::entity::AnimationClip>> {
    let assets = world.resource(asset_cache());

    let mut result: Vec<wit::entity::AnimationClip> = Vec::with_capacity(clip_urls.len());
    for clip_url in clip_urls {
        let (binders, duration, loaded, error) = match peek_loaded_clip(assets, clip_url) {
            Ok(Some(clip)) => {
                let binders: Vec<String> = clip
                    .tracks
                    .iter()
                    .map(|x| match &x.target {
                        ambient_animation::AnimationTarget::BinderId(binder) => binder.clone(),
                        ambient_animation::AnimationTarget::Entity(_entity) => String::default(),
                    })
                    .collect();

                let duration = clip.duration();
                (binders, duration, true, String::default())
            }
            Ok(None) => (Vec::default(), 0.0, false, String::default()),
            Err(err) => (Vec::default(), 0.0, false, format!("{:?}", err)),
        };
        result.push(wit::entity::AnimationClip {
            binders,
            duration,
            loaded,
            error
        });
    }

    Ok(result)
}


pub fn exists(world: &World, entity: wit::types::EntityId) -> anyhow::Result<bool> {
    Ok(world.exists(entity.from_bindgen()))
}

pub fn resources(world: &World) -> anyhow::Result<wit::types::EntityId> {
    Ok(world.resource_entity().into_bindgen())
}

pub fn synchronized_resources(world: &World) -> anyhow::Result<wit::types::EntityId> {
    Ok(world
        .synced_resource_entity()
        .context("no entity")?
        .into_bindgen())
}

pub fn persisted_resources(world: &World) -> anyhow::Result<wit::types::EntityId> {
    Ok(world
        .persisted_resource_entity()
        .context("no entity")?
        .into_bindgen())
}

pub fn in_area(
    world: &mut World,
    centre: wit::types::Vec3,
    radius: f32,
) -> anyhow::Result<Vec<wit::types::EntityId>> {
    let centre = centre.from_bindgen();
    Ok(ecs_query((translation(),))
        .iter(world, None)
        .filter_map(|(id, (pos,))| ((*pos - centre).length() < radius).then_some(id))
        .map(|id| id.into_bindgen())
        .collect())
}

pub fn get_all(world: &mut World, index: u32) -> anyhow::Result<Vec<wit::types::EntityId>> {
    let desc = match with_component_registry(|r| r.get_by_index(index)) {
        Some(c) => c,
        None => return Ok(vec![]),
    };

    Ok(
        ambient_ecs::Query::new(ambient_ecs::ArchetypeFilter::new().incl_ref(desc))
            .iter(world, None)
            .map(|ea| ea.id().into_bindgen())
            .collect(),
    )
}
