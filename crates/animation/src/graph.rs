use std::collections::{hash_map::Entry, HashMap};

use ambient_core::{asset_cache, time};
use ambient_ecs::{
    children, components,
    generated::components::core::animation::{animation_graph, blend, play_clip_from_url},
    query, Debuggable, EntityId, Networked, Store, SystemGroup, World,
};
use ambient_std::{
    asset_cache::AsyncAssetKeyExt,
    asset_url::{AnimationAssetType, ModelAssetType, TypedAssetUrl},
};

use crate::{
    AnimationClipRetargetedFromModel, AnimationOutput, AnimationRetargeting, AnimationTarget,
    AnimationTrackInterpolator, Vec3Field,
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
    time: f32,
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
        Ok(clip
            .tracks
            .iter()
            .map(|track| {
                let value = AnimationTrackInterpolator::new().value(track, time);
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

pub fn animation_graph_systems() -> SystemGroup {
    SystemGroup::new(
        "animation_graph_systems",
        vec![
            query((animation_graph(), children())).to_system(|q, world, qs, _| {
                let time = world.resource(time()).as_secs_f32();
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
        ],
    )
}
