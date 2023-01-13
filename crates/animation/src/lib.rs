use std::{
    collections::HashMap, sync::Arc, time::{Duration, SystemTime}
};

use convert_case::{Case, Casing};
use derive_more::Display;
use elements_core::{asset_cache, hierarchy::children, time};
use elements_ecs::{components, query, EntityId, SystemGroup};
use elements_model::{animation_binder, model, model_def, ModelDef};
use elements_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt}, asset_url::{AnimationAssetType, ModelAssetType, TypedAssetUrl}
};
use serde::{Deserialize, Serialize};

mod resources;
mod retargeting;

pub use resources::*;
pub use retargeting::*;

components!("animation", {
    animation_controller: AnimationController,
    animation_retargeting: AnimationRetargeting,
    /// Some animations will only work if the base pose of the character is the same as
    /// the animations base pose, so we apply the pose from the animations model to make sure they
    /// correspond
    animation_apply_base_pose: ModelDef,
    copy_animation_controller_to_children: (),
    animation_errors: String,

    /// This is a shorthand for working directly with the animation_controller
    loop_animation: TypedAssetUrl<AnimationAssetType>,
});

// Running

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationActionTime {
    Absolute { time: f32 },
    Offset { start_time: Duration, speed: f32 },
    Percentage { percentage: f32 },
}
impl AnimationActionTime {
    pub fn advance(&self, seconds: f32) -> AnimationActionTime {
        match self {
            AnimationActionTime::Absolute { time } => AnimationActionTime::Absolute { time: time + seconds },
            _ => self.clone(),
        }
    }
}
impl std::default::Default for AnimationActionTime {
    fn default() -> Self {
        Self::Absolute { time: 0.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationClipRef {
    Clip(Arc<AnimationClip>),
    FromModelAsset(TypedAssetUrl<AnimationAssetType>),
}
impl AnimationClipRef {
    // pub fn from_model(model: impl Into<ModelDef>, clip: Option<&str>) -> Self {
    //     Self::FromModelAsset(TypedAssetUrl<AnimationAssetType> { model: model.into(), retarget: None, clip: clip.map(|x| x.to_string()) })
    // }
    pub fn get_clip(
        &self,
        assets: AssetCache,
        retarget: AnimationRetargeting,
        model: Option<TypedAssetUrl<ModelAssetType>>,
    ) -> Option<Result<Arc<AnimationClip>, String>> {
        match self {
            AnimationClipRef::Clip(clip) => Some(Ok(clip.clone())),
            AnimationClipRef::FromModelAsset(def) => {
                AnimationClipRetargetedFromModel { clip: def.clone(), translation_retargeting: retarget, retarget_model: model }
                    .peek(&assets)
                    .map(|x| x.map_err(|err| format!("{err:#}")))
            }
        }
    }
}
impl From<TypedAssetUrl<AnimationAssetType>> for AnimationClipRef {
    fn from(value: TypedAssetUrl<AnimationAssetType>) -> Self {
        Self::FromModelAsset(value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationAction {
    pub clip: AnimationClipRef,
    pub time: AnimationActionTime,
    pub looping: bool,
    pub weight: f32,
}
impl AnimationAction {
    fn time(&self, time: Duration, clip: &AnimationClip) -> f32 {
        let anim_time = match self.time {
            AnimationActionTime::Offset { start_time, speed } => {
                if time < start_time {
                    -(start_time - time).as_secs_f32() * speed
                } else {
                    (time - start_time).as_secs_f32() * speed
                }
            }
            AnimationActionTime::Percentage { percentage } => percentage * clip.duration(),
            AnimationActionTime::Absolute { time } => time,
        };
        if self.looping {
            return anim_time % clip.duration();
        }
        anim_time + clip.start
    }
}

#[derive(Debug, Default, Display, Clone, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{self:?}")]
pub struct AnimationController {
    pub actions: Vec<AnimationAction>,
    /// Apply the base pose of the first animation action
    pub apply_base_pose: bool,
}
impl AnimationController {
    pub fn looping(clip: impl Into<TypedAssetUrl<AnimationAssetType>>) -> Self {
        Self::looping_with_speed(clip, 1.)
    }
    pub fn looping_with_speed(clip: impl Into<TypedAssetUrl<AnimationAssetType>>, speed: f32) -> Self {
        Self {
            actions: vec![AnimationAction {
                clip: AnimationClipRef::FromModelAsset(clip.into()),
                time: AnimationActionTime::Offset { start_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(), speed },
                looping: true,
                weight: 1.0,
            }],
            apply_base_pose: true,
        }
    }
}

#[derive(Debug)]
struct AnimationBlendOutput {
    target: EntityId,
    value: AnimationOutput,
    weight: f32,
}

pub fn animation_systems() -> SystemGroup {
    SystemGroup::new(
        "animation_systems",
        vec![
            query(loop_animation().changed()).to_system(|q, world, qs, _| {
                for (id, anim) in q.collect_cloned(world, qs) {
                    world.add_component(id, animation_controller(), AnimationController::looping(anim.clone())).unwrap();
                }
            }),
            query((animation_controller().changed(), children().changed())).incl(copy_animation_controller_to_children()).to_system(
                |q, world, qs, _| {
                    for (_, (contr, children)) in q.collect_cloned(world, qs) {
                        for c in children {
                            world.set(c, animation_controller(), contr.clone()).unwrap();
                        }
                    }
                },
            ),
            query(animation_controller().changed()).optional_changed(model()).to_system(|q, world, qs, _| {
                for (id, ctrlr) in q.collect_cloned(world, qs) {
                    world.remove_component(id, animation_errors()).unwrap();
                    if ctrlr.apply_base_pose {
                        if let Some(action) = ctrlr.actions.get(0) {
                            if let AnimationClipRef::FromModelAsset(def) = &action.clip {
                                world.add_component(id, animation_apply_base_pose(), ModelDef(def.asset_crate().unwrap().model())).unwrap();
                            }
                        }
                    }
                }
            }),
            // This exists mostly because some FBX animations have pre-rotations, and to apply them to
            // character models which don't have the same pre-rotations we need to make sure they're up to sync
            query(animation_apply_base_pose()).to_system(|q, world, qs, _| {
                let assets = world.resource(asset_cache()).clone();
                for (id, model_def) in q.collect_cloned(world, qs) {
                    let model = model_def.peek(&assets);
                    if let Some(model) = model {
                        if let Ok(model) = model {
                            model.apply_base_pose(world, id);
                        }
                        world.remove_component(id, animation_apply_base_pose()).unwrap();
                    }
                }
            }),
            query((animation_controller(), animation_binder())).excl(animation_errors()).to_system(|q, world, qs, _| {
                let assets = world.resource(asset_cache()).clone();
                let time = *world.resource(time());
                let mut outputs: HashMap<String, AnimationBlendOutput> = HashMap::new();
                let mut in_error = Vec::new();
                for (id, (controller, binder)) in q.iter(world, qs) {
                    let retaget = world.get(id, animation_retargeting()).unwrap_or(AnimationRetargeting::None);
                    let model = world.get_ref(id, model_def()).map(|def| def.0.clone()).ok();
                    // Calc
                    for action in controller.actions.iter() {
                        match action.clip.get_clip(assets.clone(), retaget, model.clone()) {
                            Some(Err(err)) => {
                                in_error.push((id, err));
                                break;
                            }
                            Some(Ok(clip)) => {
                                let anim_time = action.time(time, &clip);
                                for track in clip.tracks.iter() {
                                    let value = AnimationTrackInterpolator::new().value(track, anim_time);
                                    let key = format!(
                                        "{}_{:?}_{}_{:?}",
                                        id,
                                        track.target,
                                        track.outputs.component().get_index(),
                                        track.outputs.field()
                                    );
                                    if action.weight == 0.0 {
                                        continue;
                                    }
                                    if let Some(o) = outputs.get_mut(&key) {
                                        o.weight += action.weight;
                                        let p = action.weight / o.weight;
                                        o.value = o.value.mix(value, p);
                                    } else {
                                        outputs.insert(
                                            key.to_string(),
                                            AnimationBlendOutput {
                                                target: match &track.target {
                                                    AnimationTarget::BinderId(index) => match binder.get(index) {
                                                        Some(entity) => *entity,
                                                        None => {
                                                            continue;
                                                        }
                                                    },
                                                    AnimationTarget::Entity(entity) => *entity,
                                                },
                                                value,
                                                weight: action.weight,
                                            },
                                        );
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                }

                // Apply
                for (_, output) in outputs.into_iter() {
                    match output.value {
                        AnimationOutput::Vec3 { component, value } => {
                            world.set(output.target, component, value).ok();
                        }
                        AnimationOutput::Quat { component, value } => {
                            world.set(output.target, component, value).ok();
                        }
                        AnimationOutput::Vec3Field { component, field, value } => {
                            if let Ok(d) = world.get_mut(output.target, component) {
                                match field {
                                    Vec3Field::X => d.x = value,
                                    Vec3Field::Y => d.y = value,
                                    Vec3Field::Z => d.z = value,
                                }
                            }
                        }
                    }
                }
                for (id, err) in in_error {
                    world.add_component(id, animation_errors(), err).unwrap();
                }
            }),
        ],
    )
}

pub fn animation_bind_id_from_name(name: &str) -> String {
    let name = if let Some((_a, b)) = name.split_once(':') { b.to_string() } else { name.to_string() };
    fn normalize_name(value: &str) -> String {
        if let Some(index) = value.strip_prefix("Thumb") {
            return format!("HandThumb{index}");
        } else if let Some(index) = value.strip_prefix("Index") {
            return format!("HandIndex{index}");
        } else if let Some(index) = value.strip_prefix("Middle") {
            return format!("HandMiddle{index}");
        } else if let Some(index) = value.strip_prefix("Ring") {
            return format!("HandRing{index}");
        } else if let Some(index) = value.strip_prefix("Pinky") {
            return format!("HandPinky{index}");
        }
        match value {
            "Knee" => "Leg".to_string(),
            _ => value.to_string(),
        }
    }
    if let Some(sub) = name.strip_prefix("L_") {
        format!("Left{}", normalize_name(&sub.to_case(Case::Pascal)))
    } else if let Some(sub) = name.strip_prefix("R_") {
        format!("Right{}", normalize_name(&sub.to_case(Case::Pascal)))
    } else {
        let name = name.to_case(Case::Pascal);
        if name.contains("Armature") {
            "Armature".to_string()
        } else {
            name
        }
    }
}

#[test]
fn test_animation() {
    use elements_core::transform::translation;
    use glam::vec3;
    let mut int = AnimationTrackInterpolator::new();
    let track = AnimationTrack {
        target: AnimationTarget::BinderId("".to_string()),
        inputs: vec![0., 1.],
        outputs: AnimationOutputs::Vec3 { component: translation(), data: vec![vec3(0.5, 0., 0.), vec3(1., 0., 0.)] },
    };
    assert_eq!(0.5, int.value(&track, -0.5).as_vec3_value().unwrap().x);
    assert_eq!(0.5, int.value(&track, 0.).as_vec3_value().unwrap().x);
    assert_eq!(0.75, int.value(&track, 0.5).as_vec3_value().unwrap().x);
    assert_eq!(1., int.value(&track, 1.).as_vec3_value().unwrap().x);
    assert_eq!(1., int.value(&track, 1.5).as_vec3_value().unwrap().x);
}
