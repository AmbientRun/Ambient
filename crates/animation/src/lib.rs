use ambient_core::{asset_cache, dtime, hierarchy::children, time};
use ambient_ecs::{
    components, query, query_mut, Debuggable, EntityId, MakeDefault, Networked, Store, SystemGroup,
};
use ambient_model::{animation_binder, model, model_from_url, ModelFromUrl};
use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKeyExt},
    asset_url::{AnimationAssetType, ModelAssetType, TypedAssetUrl},
};
use ambient_sys::time::SystemTime;
use convert_case::{Case, Casing};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};

mod resources;
mod retargeting;

pub use resources::*;
pub use retargeting::*;

components!("animation", {
    @[Debuggable, Networked, Store]
    animation_controller: AnimationController,
    @[MakeDefault ,Debuggable, Networked, Store]
    animation_retargeting: AnimationRetargeting,
    /// Some animations will only work if the base pose of the character is the same as
    /// the animations base pose, so we apply the pose from the animations model to make sure they
    /// correspond
    @[Debuggable, Networked, Store]
    animation_apply_base_pose: ModelFromUrl,
    @[Debuggable, Networked, Store]
    copy_animation_controller_to_children: (),
    @[Debuggable, Networked, Store]
    animation_errors: String,

    /// This is a shorthand for working directly with the animation_controller
    @[MakeDefault,  Debuggable, Networked, Store]
    loop_animation: TypedAssetUrl<AnimationAssetType>,

    @[Debuggable, Networked, Store]
    animation_stack: Vec<AnimationActionStack>,
    @[Debuggable, Networked, Store]
    animation_binder_mask: Vec<String>,
    @[Debuggable, Networked, Store]
    animation_binder_weights: Vec<Vec<f32>>,
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
            AnimationActionTime::Absolute { time } => AnimationActionTime::Absolute {
                time: time + seconds,
            },
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
        assets: &AssetCache,
        retarget: AnimationRetargeting,
        model: Option<TypedAssetUrl<ModelAssetType>>,
    ) -> Option<Result<Arc<AnimationClip>, String>> {
        match self {
            AnimationClipRef::Clip(clip) => Some(Ok(clip.clone())),
            AnimationClipRef::FromModelAsset(def) => AnimationClipRetargetedFromModel {
                clip: def.clone(),
                translation_retargeting: retarget,
                retarget_model: model,
            }
            .peek(assets)
            .map(|x| x.map_err(|err| format!("{err:#}"))),
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
}

fn anim_time(
    time: &Duration,
    action_time: &AnimationActionTime,
    clip: &AnimationClip,
    looping: bool,
) -> f32 {
    let anim_time = match *action_time {
        AnimationActionTime::Offset { start_time, speed } => {
            if *time < start_time {
                -(start_time - *time).as_secs_f32() * speed
            } else {
                (*time - start_time).as_secs_f32() * speed
            }
        }
        AnimationActionTime::Percentage { percentage } => percentage * clip.duration(),
        AnimationActionTime::Absolute { time } => time,
    };
    if looping {
        return anim_time % clip.duration();
    }
    anim_time + clip.start
}

impl AnimationAction {
    fn sample_tracks(
        &self,
        time: &Duration,
        action_time: &AnimationActionTime,
        assets: &AssetCache,
        retarget: AnimationRetargeting,
        model: Option<TypedAssetUrl<ModelAssetType>>,
        binder: &HashMap<String, EntityId>,
        outputs: &mut HashMap<(EntityId, u32, Option<Vec3Field>), AnimationOutput>,
    ) -> Result<(), String> {
        let clip = match self.clip.get_clip(assets, retarget, model.clone()) {
            Some(Err(err)) => return Err(err),
            Some(Ok(clip)) => clip,
            None => return Ok(()),
        };

        let anim_time = anim_time(time, action_time, &clip, self.looping);

        for track in clip.tracks.iter() {
            let entity = match &track.target {
                AnimationTarget::BinderId(index) => match binder.get(index) {
                    Some(entity) => *entity,
                    None => {
                        continue;
                    }
                },
                AnimationTarget::Entity(entity) => *entity,
            };
            let component = track.outputs.component().index();
            let field = track.outputs.field();
            let value = AnimationTrackInterpolator::new().value(track, anim_time);
            outputs.insert((entity, component, field), value);
        }
        Ok(())
    }

    pub fn start(&mut self, speed: f32) {
        self.time = AnimationActionTime::Offset {
            start_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            speed,
        };
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
    pub fn looping_with_speed(
        clip: impl Into<TypedAssetUrl<AnimationAssetType>>,
        speed: f32,
    ) -> Self {
        Self {
            actions: vec![AnimationAction {
                clip: AnimationClipRef::FromModelAsset(clip.into()),
                time: AnimationActionTime::Offset {
                    start_time: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap(),
                    speed,
                },
                looping: true,
            }],
            apply_base_pose: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationActionStack {
    Start {
        action_index: u32,
        speed: f32,
    },
    Sample {
        action_index: u32,
    },
    SampleAbsolute {
        action_index: u32,
        time_absolute: f32,
    },
    SamplePercentage {
        action_index: u32,
        time_percentage: f32,
    },
    Interpolate {
        weight: f32,
    },
    Blend {
        weight: f32,
        mask: u32,
    },
    Transition {
        weight: f32,
        duration: f32,
    },
}

pub fn animation_systems() -> SystemGroup {
    SystemGroup::new(
        "animation_systems",
        vec![
            query(loop_animation().changed()).to_system(|q, world, qs, _| {
                for (id, anim) in q.collect_cloned(world, qs) {
                    world
                        .add_component(
                            id,
                            animation_controller(),
                            AnimationController::looping(anim.clone()),
                        )
                        .unwrap();
                }
            }),
            query((animation_controller().changed(), children().changed()))
                .incl(copy_animation_controller_to_children())
                .to_system(|q, world, qs, _| {
                    for (_, (contr, children)) in q.collect_cloned(world, qs) {
                        for c in children {
                            world.set(c, animation_controller(), contr.clone()).unwrap();
                        }
                    }
                }),
            query(animation_controller().changed())
                .optional_changed(model())
                .to_system(|q, world, qs, _| {
                    for (id, ctrlr) in q.collect_cloned(world, qs) {
                        world.remove_component(id, animation_errors()).unwrap();
                        if ctrlr.apply_base_pose {
                            if let Some(action) = ctrlr.actions.get(0) {
                                if let AnimationClipRef::FromModelAsset(def) = &action.clip {
                                    world
                                        .add_component(
                                            id,
                                            animation_apply_base_pose(),
                                            ModelFromUrl(def.model_crate().unwrap().model()),
                                        )
                                        .unwrap();
                                }
                            }
                        }
                        if !world.has_component(id, animation_binder_mask()) {
                            world
                                .add_component(id, animation_binder_mask(), vec![])
                                .unwrap();
                        }

                        if !world.has_component(id, animation_binder_weights()) {
                            world
                                .add_component(id, animation_binder_weights(), vec![])
                                .unwrap();
                        }

                        if !world.has_component(id, animation_stack()) && !ctrlr.actions.is_empty()
                        {
                            world
                                .add_component(
                                    id,
                                    animation_stack(),
                                    vec![AnimationActionStack::Sample { action_index: 0 }],
                                )
                                .unwrap();
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
                        world
                            .remove_component(id, animation_apply_base_pose())
                            .unwrap();
                    }
                }
            }),
            query_mut(
                (animation_stack(), animation_controller()),
                (
                    animation_binder(),
                    animation_binder_mask(),
                    animation_binder_weights(),
                ),
            )
            .excl(animation_errors())
            .to_system(|q, world, mut qs, _| {
                let assets = world.resource(asset_cache()).clone();
                let time = *world.resource(time());
                let delta_time = *world.resource(dtime());
                let mut in_error = Vec::new();

                let mut buffers = Vec::new();
                let mut samples = Vec::new();
                let mut entity_weight_index: HashMap<EntityId, usize> = HashMap::new();
                let mut results = Vec::new();

                let mut optional_components: HashMap<
                    EntityId,
                    (
                        Option<AnimationRetargeting>,
                        Option<TypedAssetUrl<ModelAssetType>>,
                    ),
                > = HashMap::new();

                if let Some(qs) = &mut qs {
                    for (id, ..) in q.iter(world, Some(*qs)) {
                        optional_components.insert(id, (None, None));
                    }
                }

                for (&id, (retarget, model)) in optional_components.iter_mut() {
                    *retarget = world.get(id, animation_retargeting()).ok();
                    *model = world
                        .get_ref(id, model_from_url())
                        .ok()
                        .and_then(|def| TypedAssetUrl::parse(def).ok());
                }

                for (id, (stack, controller), (binder, binder_mask, binder_weights)) in
                    q.iter(world, qs)
                {
                    let (retarget, model) =
                        optional_components.get(&id).cloned().unwrap_or_default();
                    let retarget = retarget.unwrap_or(AnimationRetargeting::None);
                    entity_weight_index.clear();
                    samples.clear();

                    let mut completed = None;
                    type AnimationBuffer =
                        HashMap<(EntityId, u32, Option<Vec3Field>), AnimationOutput>;

                    let mut current_transition: Option<(AnimationBuffer, f32)> = None;
                    for (op_index, operation) in stack.iter_mut().enumerate() {
                        match operation {
                            &mut AnimationActionStack::Start {
                                action_index,
                                speed,
                            } => {
                                let mut tracks =
                                    buffers.pop().unwrap_or_else(|| HashMap::with_capacity(128));
                                tracks.clear();
                                if let Some(action) =
                                    controller.actions.get_mut(action_index as usize)
                                {
                                    action.start(speed);

                                    if let Err(err) = action.sample_tracks(
                                        &time,
                                        &action.time,
                                        &assets,
                                        retarget,
                                        model.clone(),
                                        binder,
                                        &mut tracks,
                                    ) {
                                        in_error.push((id, err));
                                        break;
                                    }
                                } else {
                                    in_error.push((id, "Missing Blend Action".to_owned()));
                                    break;
                                }
                                samples.push(tracks);
                                *operation = AnimationActionStack::Sample { action_index };
                            }
                            &mut AnimationActionStack::Sample { action_index } => {
                                let mut tracks =
                                    buffers.pop().unwrap_or_else(|| HashMap::with_capacity(128));
                                tracks.clear();
                                if let Some(action) = controller.actions.get(action_index as usize)
                                {
                                    if let Err(err) = action.sample_tracks(
                                        &time,
                                        &action.time,
                                        &assets,
                                        retarget,
                                        model.clone(),
                                        binder,
                                        &mut tracks,
                                    ) {
                                        in_error.push((id, err));
                                        break;
                                    }
                                } else {
                                    in_error.push((id, "Missing Blend Action".to_owned()));
                                    break;
                                }
                                samples.push(tracks);
                            }
                            &mut AnimationActionStack::SampleAbsolute {
                                action_index,
                                time_absolute,
                            } => {
                                let mut tracks =
                                    buffers.pop().unwrap_or_else(|| HashMap::with_capacity(128));
                                tracks.clear();
                                if let Some(action) = controller.actions.get(action_index as usize)
                                {
                                    if let Err(err) = action.sample_tracks(
                                        &time,
                                        &AnimationActionTime::Absolute {
                                            time: time_absolute,
                                        },
                                        &assets,
                                        retarget,
                                        model.clone(),
                                        binder,
                                        &mut tracks,
                                    ) {
                                        in_error.push((id, err));
                                        break;
                                    }
                                } else {
                                    in_error.push((id, "Missing Blend Action".to_owned()));
                                    break;
                                }
                                samples.push(tracks);
                            }
                            &mut AnimationActionStack::SamplePercentage {
                                action_index,
                                time_percentage,
                            } => {
                                let mut tracks =
                                    buffers.pop().unwrap_or_else(|| HashMap::with_capacity(128));
                                tracks.clear();
                                if let Some(action) = controller.actions.get(action_index as usize)
                                {
                                    if let Err(err) = action.sample_tracks(
                                        &time,
                                        &AnimationActionTime::Percentage {
                                            percentage: time_percentage,
                                        },
                                        &assets,
                                        retarget,
                                        model.clone(),
                                        binder,
                                        &mut tracks,
                                    ) {
                                        in_error.push((id, err));
                                        break;
                                    }
                                } else {
                                    in_error.push((id, "Missing Blend Action".to_owned()));
                                    break;
                                }
                                samples.push(tracks);
                            }
                            &mut AnimationActionStack::Interpolate { weight } => {
                                let rhs = samples.pop();
                                let lhs = samples.pop();

                                match (lhs, rhs) {
                                    (Some(mut lhs), Some(mut rhs)) => {
                                        for (key, target) in rhs.drain() {
                                            if let Some(source) = lhs.get_mut(&key) {
                                                *source = source.mix(target, weight);
                                            } else {
                                                lhs.insert(key, target);
                                            }
                                        }
                                        buffers.push(rhs);
                                        samples.push(lhs);
                                    }
                                    _ => {
                                        in_error.push((id, "Invalid blend stack".to_owned()));
                                        break;
                                    }
                                }
                            }
                            &mut AnimationActionStack::Blend { weight, mask } => {
                                if entity_weight_index.is_empty() {
                                    for (index, key) in binder_mask.iter().enumerate() {
                                        if let Some(entity) = binder.get(key) {
                                            entity_weight_index.insert(*entity, index);
                                        }
                                    }
                                }
                                let rhs = samples.pop();
                                let lhs = samples.pop();

                                match (lhs, rhs) {
                                    (Some(mut lhs), Some(mut rhs)) => {
                                        if let Some(mask) = binder_weights.get(mask as usize) {
                                            for (key, target) in rhs.drain() {
                                                if let Some(&mask_weight) = entity_weight_index
                                                    .get(&key.0)
                                                    .and_then(|&x| mask.get(x))
                                                {
                                                    if mask_weight == 0.0 {
                                                        continue;
                                                    }
                                                    if let Some(source) = lhs.get_mut(&key) {
                                                        *source = source
                                                            .mix(target, mask_weight * weight);
                                                    } else {
                                                        lhs.insert(key, target);
                                                    }
                                                }
                                            }
                                        }
                                        buffers.push(rhs);
                                        samples.push(lhs);
                                    }
                                    _ => {
                                        in_error.push((id, "Invalid blend stack".to_owned()));
                                        break;
                                    }
                                }
                            }
                            AnimationActionStack::Transition { weight, duration } => {
                                if *duration > 0.0 {
                                    *weight += delta_time / *duration;
                                } else {
                                    *weight = 1.0;
                                }

                                if *weight >= 1.0 {
                                    buffers.append(&mut samples);
                                    completed = Some(op_index + 1);
                                } else {
                                    if let Some(mut rhs) = samples.pop() {
                                        if let Some((mut lhs, t)) = current_transition.take() {
                                            for (key, target) in rhs.drain() {
                                                if let Some(source) = lhs.get_mut(&key) {
                                                    *source = source.mix(target, t);
                                                } else {
                                                    lhs.insert(key, target);
                                                }
                                            }
                                            buffers.push(rhs);
                                            current_transition = Some((lhs, *weight));
                                        } else {
                                            current_transition = Some((rhs, *weight));
                                        }
                                    } else {
                                        in_error.push((id, "Invalid blend stack".to_owned()));
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(count) = completed {
                        stack.drain(0..count);
                    }

                    if let Some(mut rhs) = samples.pop() {
                        if let Some((mut lhs, t)) = current_transition.take() {
                            for (key, target) in rhs.drain() {
                                if let Some(source) = lhs.get_mut(&key) {
                                    *source = source.mix(target, t);
                                } else {
                                    lhs.insert(key, target);
                                }
                            }
                            buffers.push(rhs);
                            results.push(lhs);
                        } else {
                            results.push(rhs);
                        }
                    } else {
                        in_error.push((id, "Invalid blend stack".to_owned()));
                    }

                    buffers.append(&mut samples);
                }

                for result in results.into_iter() {
                    for ((target, ..), value) in result.into_iter() {
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

                for (id, err) in in_error {
                    world.add_component(id, animation_errors(), err).unwrap();
                }
            }),
        ],
    )
}

pub fn animation_bind_id_from_name(name: &str) -> String {
    let name = if let Some((_a, b)) = name.split_once(':') {
        b.to_string()
    } else {
        name.to_string()
    };
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
    use ambient_core::transform::{self, translation};
    use glam::vec3;

    ambient_ecs::init_components();
    transform::init_components();

    let mut int = AnimationTrackInterpolator::new();
    let track = AnimationTrack {
        target: AnimationTarget::BinderId("".to_string()),
        inputs: vec![0., 1.],
        outputs: AnimationOutputs::Vec3 {
            component: translation(),
            data: vec![vec3(0.5, 0., 0.), vec3(1., 0., 0.)],
        },
    };
    assert_eq!(0.5, int.value(&track, -0.5).as_vec3_value().unwrap().x);
    assert_eq!(0.5, int.value(&track, 0.).as_vec3_value().unwrap().x);
    assert_eq!(0.75, int.value(&track, 0.5).as_vec3_value().unwrap().x);
    assert_eq!(1., int.value(&track, 1.).as_vec3_value().unwrap().x);
    assert_eq!(1., int.value(&track, 1.5).as_vec3_value().unwrap().x);
}
