use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use ambient_core::{asset_cache, async_ecs::async_run, runtime, time};
use ambient_ecs::{
    children, components,
    generated::components::core::animation::{
        animation_graph, apply_animation_graph, blend, clip_duration, freeze_at_percentage,
        freeze_at_time, looping, mask_bind_ids, mask_weights, play_clip_from_url, speed,
        start_time,
    },
    query, Debuggable, EntityId, Networked, Store, SystemGroup, World,
};
use ambient_model::animation_binder;
use ambient_std::{
    asset_cache::AsyncAssetKeyExt,
    asset_url::{AbsAssetUrl, AnimationAssetType, ModelAssetType, TypedAssetUrl},
};

use crate::{
    animation_errors, AnimationClipFromUrl, AnimationClipRetargetedFromModel, AnimationOutput,
    AnimationRetargeting, AnimationTarget, AnimationTrackInterpolator, Vec3Field,
};

components!("animation", {
    @[Debuggable]
    animation_output: HashMap<AnimationOutputKey, AnimationOutput>,
    @[Debuggable]
    mask: HashMap<String, f32>,
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
        Ok(clip
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
            .collect())
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
    world: &mut World,
    binder: HashMap<String, EntityId>,
    outputs: HashMap<AnimationOutputKey, AnimationOutput>,
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
                world.set(target, component, value).ok();
            }
            AnimationOutput::Quat { component, value } => {
                world.set(target, component, value).ok();
            }
            AnimationOutput::Vec3Field {
                component,
                field,
                value,
            } => {
                if let Ok(d) = world.get_mut(target, component) {
                    match field {
                        Vec3Field::X => d.x = value,
                        Vec3Field::Y => d.y = value,
                        Vec3Field::Z => d.z = value,
                    }
                }
            }
        }
    }
}

pub fn animation_graph_systems() -> SystemGroup {
    SystemGroup::new(
        "animation_graph_systems",
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
                        world.add_component(id, mask(), mask_map);
                    }
                }),
            query((animation_graph(), children())).to_system(|q, world, qs, _| {
                let time = world.resource(time()).clone();
                for (id, (_, children)) in q.collect_cloned(world, qs) {
                    let mut errors = Default::default();
                    let output = sample_animation_node(
                        world,
                        children[0],
                        time,
                        AnimationRetargeting::None,
                        &None,
                        &mut errors,
                    );
                    world.add_component(id, animation_output(), output);
                    if errors.len() > 0 {
                        world.add_component(id, animation_errors(), errors.join("\n"));
                    } else if world.has_component(id, animation_errors()) {
                        world.remove_component(id, animation_errors());
                    }
                }
            }),
            query((apply_animation_graph(), animation_binder())).to_system(|q, world, qs, _| {
                for (id, (anim_graph_id, binder)) in q.collect_cloned(world, qs) {
                    let outputs = world.get_cloned(anim_graph_id, animation_output()).unwrap();
                    apply_animation_outputs_to_entity(world, binder, outputs);
                }
            }),
        ],
    )
}
