use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use ambient_core::{asset_cache, async_ecs::async_run, runtime, time};
use ambient_ecs::{
    children, components,
    generated::components::core::animation::{
        animation_graph, apply_animation_graph, blend, clip_duration, looping, play_clip_from_url,
    },
    query, Debuggable, EntityId, Networked, Store, SystemGroup, World,
};
use ambient_model::animation_binder;
use ambient_std::{
    asset_cache::AsyncAssetKeyExt,
    asset_url::{AbsAssetUrl, AnimationAssetType, ModelAssetType, TypedAssetUrl},
};

use crate::{
    AnimationClipFromUrl, AnimationClipRetargetedFromModel, AnimationOutput, AnimationRetargeting,
    AnimationTarget, AnimationTrackInterpolator, Vec3Field,
};

components!("animation", {
    @[Debuggable]
    animation_output: HashMap<AnimationOutputKey, AnimationOutput>,
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
    mut time: f64,
    retargeting: AnimationRetargeting,
    model: &Option<TypedAssetUrl<ModelAssetType>>,
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
        if world.get(node, looping()).unwrap_or(false) {
            time = time % clip.duration() as f64;
        }
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
        let mut output = sample_animation_node(world, children[0], time, retargeting, model)?;
        let right = sample_animation_node(world, children[1], time, retargeting, model)?;
        for (key, value) in right.into_iter() {
            match output.entry(key) {
                Entry::Occupied(mut o) => {
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
            query((animation_graph(), children())).to_system(|q, world, qs, _| {
                let time = world.resource(time()).as_secs_f64();
                for (id, (_, children)) in q.collect_cloned(world, qs) {
                    let output = sample_animation_node(
                        world,
                        children[0],
                        time,
                        AnimationRetargeting::None,
                        &None,
                    );
                    match output {
                        Ok(output) => {
                            world.add_component(id, animation_output(), output);
                        }
                        Err(err) => {
                            log::error!("Animation graph error: {:?}", err)
                        }
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
