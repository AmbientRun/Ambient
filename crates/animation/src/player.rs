use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use ambient_core::{
    asset_cache, async_ecs::async_run, hierarchy::despawn_recursive, runtime, time,
};
use ambient_ecs::{
    children, components,
    generated::components::core::animation::{
        animation_errors, animation_player, apply_animation_player, apply_base_pose, blend,
        clip_duration, freeze_at_percentage, freeze_at_time, looping, mask_bind_ids, mask_weights,
        play_clip_from_url, ref_count, retarget_animation_scaled, retarget_model_from_url, speed,
        start_time,
    },
    parent, query, ComponentDesc, Debuggable, EntityId, SystemGroup, World,
};
use ambient_model::{animation_binder, ModelFromUrl};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt},
    asset_url::{AbsAssetUrl, AnimationAssetType, ModelAssetType, TypedAssetUrl},
};
use anyhow::Context;
use glam::{Quat, Vec3};

use crate::{
    AnimationClipFromUrl, AnimationClipRetargetedFromModel, AnimationOutput, AnimationRetargeting,
    AnimationTarget, AnimationTrackInterpolator, Vec3Field,
};

components!("animation", {
    @[Debuggable]
    animation_output: HashMap<AnimationOutputKey, AnimationOutput>,
    @[Debuggable]
    mask: HashMap<String, f32>,
    cached_base_pose: HashMap<AnimationOutputKey, AnimationOutput>,

});

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AnimationOutputKey {
    target: AnimationTarget,
    component: u32,
    field: Option<Vec3Field>,
}

fn sample_animation_node(
    world: &World,
    node: EntityId,
    time: Duration,
    retargeting: AnimationRetargeting,
    model: &Option<TypedAssetUrl<ModelAssetType>>,
    errors: &mut Vec<String>,
) -> HashMap<AnimationOutputKey, AnimationOutput> {
    match sample_animation_node_inner(world, node, time, retargeting, model, errors) {
        Ok(val) => val,
        Err(err) => {
            errors.push(format!("Node {}: {:?}", node, err));
            Default::default()
        }
    }
}
fn sample_animation_node_inner(
    world: &World,
    node: EntityId,
    time: Duration,
    retargeting: AnimationRetargeting,
    model: &Option<TypedAssetUrl<ModelAssetType>>,
    errors: &mut Vec<String>,
) -> anyhow::Result<HashMap<AnimationOutputKey, AnimationOutput>> {
    if let Ok(url) = world.get_cloned(node, play_clip_from_url()) {
        let clip = AnimationClipRetargetedFromModel {
            clip: TypedAssetUrl::<AnimationAssetType>::parse(url)?,
            translation_retargeting: retargeting,
            retarget_model: model.clone(),
        }
        .peek(world.resource(asset_cache()));
        let clip = match clip {
            Some(value) => value?,
            None => return Ok(Default::default()),
        };
        let time = if let Ok(freeze_at_time) = world.get(node, freeze_at_time()) {
            freeze_at_time as f64
        } else if let Ok(freeze_at_percentage) = world.get(node, freeze_at_percentage()) {
            (freeze_at_percentage * clip.duration()) as f64
        } else {
            let mut time = match world.get_cloned(node, start_time()) {
                Ok(st) => (time - st).as_secs_f64(),
                Err(_) => time.as_secs_f64(),
            };
            let speed = world.get_cloned(node, speed()).unwrap_or(1.);
            if world.get(node, looping()).unwrap_or(false) {
                time = time % clip.duration() as f64;
            }
            time * speed as f64
        };
        let mut output: HashMap<AnimationOutputKey, AnimationOutput> = clip
            .tracks
            .iter()
            .map(|track| {
                let value = AnimationTrackInterpolator::new().value(track, time as f32);
                let key = AnimationOutputKey {
                    target: track.target.clone(),
                    component: track.outputs.component().index(),
                    field: track.outputs.field(),
                };
                (key, value)
            })
            .collect();
        if let Ok(base_pose) = world.get_ref(node, cached_base_pose()) {
            for (key, value) in base_pose.into_iter() {
                if !output.contains_key(&key) {
                    output.insert(key.clone(), value.clone());
                }
            }
        }
        Ok(output)
    } else if let Ok(blend_weight) = world.get_cloned(node, blend()) {
        let children = world.get_cloned(node, children())?;
        if children.len() != 2 {
            anyhow::bail!("Animation blend node needs to have exactly two children");
        }
        let mut output =
            sample_animation_node(world, children[0], time, retargeting, model, errors);
        let right = sample_animation_node(world, children[1], time, retargeting, model, errors);
        let mask = world.get_cloned(node, mask()).ok();
        for (key, value) in right.into_iter() {
            match output.entry(key.clone()) {
                Entry::Occupied(mut o) => {
                    let mut blend_weight = blend_weight;
                    if let Some(mask) = &mask {
                        if let AnimationTarget::BinderId(bind_id) = &key.target {
                            if let Some(weight) = mask.get(bind_id) {
                                blend_weight = *weight;
                            }
                        }
                    }
                    let left = o.get_mut();
                    *left = left.mix(value, blend_weight);
                }
                Entry::Vacant(v) => {
                    v.insert(value);
                }
            }
        }
        Ok(output)
    } else {
        anyhow::bail!("Node is not a proper animation node")
    }
}

fn apply_animation_outputs_to_entity(
    world: &World,
    binder: &HashMap<String, EntityId>,
    outputs: &HashMap<AnimationOutputKey, AnimationOutput>,
) {
    for (key, value) in outputs.into_iter() {
        let target = match &key.target {
            AnimationTarget::BinderId(id) => match binder.get(id) {
                Some(id) => *id,
                None => {
                    continue;
                }
            },
            AnimationTarget::Entity(id) => *id,
        };
        match value {
            AnimationOutput::Vec3 { component, value } => {
                if let Ok(v) = world.get_mut_unsafe(target, *component) {
                    *v = *value;
                }
            }
            AnimationOutput::Quat { component, value } => {
                if let Ok(v) = world.get_mut_unsafe(target, *component) {
                    *v = *value;
                }
            }
            AnimationOutput::Vec3Field {
                component,
                field,
                value,
            } => {
                if let Ok(d) = world.get_mut_unsafe(target, *component) {
                    match field {
                        Vec3Field::X => d.x = *value,
                        Vec3Field::Y => d.y = *value,
                        Vec3Field::Z => d.z = *value,
                    }
                }
            }
        }
    }
}

pub fn animation_player_systems() -> SystemGroup {
    SystemGroup::new(
        "animation_player_systems",
        vec![
            query(play_clip_from_url().changed()).to_system(|q, world, qs, _| {
                // Set clip_duration for all play_clip_from_url nodes
                let runtime = world.resource(runtime()).clone();
                for (id, url) in q.collect_cloned(world, qs) {
                    let async_run = world.resource(async_run()).clone();
                    let assets = world.resource(asset_cache()).clone();
                    let url = match AbsAssetUrl::parse(url) {
                        Ok(val) => val,
                        Err(_) => continue,
                    };
                    runtime.spawn(async move {
                        if let Ok(clip) = AnimationClipFromUrl::new(url, true).get(&assets).await {
                            let duration = clip.duration();
                            async_run.run(move |world| {
                                world.add_component(id, clip_duration(), duration).ok();
                            });
                        }
                    });
                }
            }),
            query(play_clip_from_url().changed())
                .incl(apply_base_pose())
                .to_system(|q, world, qs, _| {
                    for (id, url) in q.collect_cloned(world, qs) {
                        if let Ok(base_pose) = build_base_pose(world.resource(asset_cache()), &url)
                        {
                            world.add_component(id, cached_base_pose(), base_pose).ok();
                        }
                    }
                }),
            query(mask_bind_ids().changed())
                .optional_changed(mask_weights())
                .to_system(|q, world, qs, _| {
                    for (id, bind_ids) in q.collect_cloned(world, qs) {
                        let mut mask_map = HashMap::new();
                        let mask_weights = world.get_cloned(id, mask_weights()).unwrap_or_default();
                        for (i, bind_id) in bind_ids.iter().enumerate() {
                            mask_map.insert(
                                bind_id.clone(),
                                mask_weights.get(i).map(|x| *x).unwrap_or(0.),
                            );
                        }
                        world.add_component(id, mask(), mask_map).ok();
                    }
                }),
            query((animation_player(), children())).to_system(|q, world, qs, _| {
                let time = world.resource(time()).clone();
                for (id, (_, children)) in q.collect_cloned(world, qs) {
                    let retarget_model = world
                        .get_cloned(id, retarget_model_from_url())
                        .ok()
                        .and_then(|x| TypedAssetUrl::parse(x).ok());
                    let retarget_animation_scaled = world.get(id, retarget_animation_scaled()).ok();
                    let mut errors = Default::default();
                    let output = sample_animation_node(
                        world,
                        children[0],
                        time,
                        if retarget_model.is_some() {
                            if let Some(hip) = retarget_animation_scaled {
                                AnimationRetargeting::AnimationScaled { normalize_hip: hip }
                            } else {
                                AnimationRetargeting::Skeleton
                            }
                        } else {
                            AnimationRetargeting::None
                        },
                        &retarget_model,
                        &mut errors,
                    );
                    world.add_component(id, animation_output(), output).ok();
                    if errors.len() > 0 {
                        world.add_component(id, animation_errors(), errors).ok();
                    } else if world.has_component(id, animation_errors()) {
                        world.remove_component(id, animation_errors()).ok();
                    }
                }
            }),
            query((apply_animation_player(), animation_binder())).to_system(|q, world, qs, _| {
                for (_, (anim_player_id, binder)) in q.iter(world, qs) {
                    if let Ok(outputs) = world.get_ref(*anim_player_id, animation_output()) {
                        apply_animation_outputs_to_entity(world, binder, outputs);
                    }
                }
            }),
            // Cleanup
            query(ref_count().changed())
                .excl(parent())
                .to_system(|q, world, qs, _| {
                    for (id, count) in q.collect_cloned(world, qs) {
                        if count == 0 {
                            despawn_recursive(world, id);
                        }
                    }
                }),
        ],
    )
}

fn build_base_pose(
    assets: &AssetCache,
    clip_url: &str,
) -> anyhow::Result<HashMap<AnimationOutputKey, AnimationOutput>> {
    let clip_url = TypedAssetUrl::<AnimationAssetType>::parse(clip_url)?;
    let model_url = clip_url
        .model_crate()
        .context("Failed to get model crate")?
        .model();
    if let Some(Ok(model)) = ModelFromUrl(model_url).peek(assets) {
        Ok(model
            .build_base_pose()
            .into_iter()
            .flat_map(|(bind_id, entity)| {
                entity.into_iter().map(move |entry| {
                    let desc: ComponentDesc = (*entry).clone();
                    let output = if let Some(value) = entry.try_downcast_ref::<Vec3>() {
                        AnimationOutput::Vec3 {
                            component: desc.try_into().unwrap(),
                            value: *value,
                        }
                    } else if let Some(value) = entry.try_downcast_ref::<Quat>() {
                        AnimationOutput::Quat {
                            component: desc.try_into().unwrap(),
                            value: *value,
                        }
                    } else {
                        todo!()
                    };
                    let key = AnimationOutputKey {
                        target: AnimationTarget::BinderId(bind_id.clone()),
                        component: desc.index(),
                        field: None,
                    };
                    (key, output)
                })
            })
            .collect())
    } else {
        anyhow::bail!("No anim yet")
    }
}
