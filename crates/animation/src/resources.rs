use std::{collections::HashMap, f32::consts::PI, ops::Neg};

use ambient_core::transform::{euler_rotation, rotation, scale, translation};
use ambient_ecs::{Component, ComponentDesc, EntityId};
use ambient_std::{download_asset::BincodeFromUrl, math::mix};
use glam::{EulerRot, Quat, Vec3};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AnimationOutput {
    Vec3 {
        component: Component<glam::Vec3>,
        value: glam::Vec3,
    },
    Quat {
        component: Component<glam::Quat>,
        value: glam::Quat,
    },
    Vec3Field {
        component: Component<glam::Vec3>,
        field: Vec3Field,
        value: f32,
    },
}
impl AnimationOutput {
    pub fn mix(&self, value: AnimationOutput, p: f32) -> Self {
        match (self, value) {
            (
                AnimationOutput::Vec3 { value: left, .. },
                AnimationOutput::Vec3 {
                    value: right,
                    component,
                },
            ) => AnimationOutput::Vec3 {
                component,
                value: left.lerp(right, p),
            },

            (
                AnimationOutput::Quat { value: left, .. },
                AnimationOutput::Quat {
                    value: right,
                    component,
                },
            ) => {
                let d = left.dot(right);
                AnimationOutput::Quat {
                    component,
                    value: if d >= 0. {
                        left.slerp(right, p)
                    } else {
                        left.neg().slerp(right, p)
                    },
                }
            }

            (
                AnimationOutput::Vec3Field { value: left, .. },
                AnimationOutput::Vec3Field {
                    value: right,
                    field,
                    component,
                },
            ) => AnimationOutput::Vec3Field {
                component,
                field,
                value: mix(*left, right, p),
            },

            _ => unreachable!(),
        }
    }
    pub fn as_vec3_value(&self) -> Option<&Vec3> {
        match self {
            AnimationOutput::Vec3 { value, .. } => Some(value),
            _ => None,
        }
    }
    pub fn as_field_value(&self) -> Option<f32> {
        match self {
            AnimationOutput::Vec3Field { value, .. } => Some(*value),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Vec3Field {
    X,
    Y,
    Z,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationOutputs {
    Vec3 {
        component: Component<glam::Vec3>,
        data: Vec<glam::Vec3>,
    },
    Quat {
        component: Component<glam::Quat>,
        data: Vec<glam::Quat>,
    },
    Vec3Field {
        component: Component<glam::Vec3>,
        field: Vec3Field,
        data: Vec<f32>,
    },
}
impl AnimationOutputs {
    pub fn component(&self) -> ComponentDesc {
        match self {
            AnimationOutputs::Vec3 { component, .. } => component.desc(),
            AnimationOutputs::Quat { component, .. } => component.desc(),
            AnimationOutputs::Vec3Field { component, .. } => component.desc(),
        }
    }
    pub fn field(&self) -> Option<Vec3Field> {
        match self {
            AnimationOutputs::Vec3Field { field, .. } => Some(*field),
            _ => None,
        }
    }
    pub fn field_values_mut(&mut self) -> Option<&mut Vec<f32>> {
        match self {
            AnimationOutputs::Vec3Field { data, .. } => Some(data),
            _ => None,
        }
    }
    pub fn value(&self, index: usize) -> AnimationOutput {
        match self {
            AnimationOutputs::Vec3 { data, component } => AnimationOutput::Vec3 {
                component: *component,
                value: data[index],
            },
            AnimationOutputs::Quat { data, component } => AnimationOutput::Quat {
                component: *component,
                value: data[index],
            },
            AnimationOutputs::Vec3Field {
                data,
                component,
                field,
            } => AnimationOutput::Vec3Field {
                component: *component,
                field: *field,
                value: data[index],
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimationTarget {
    BinderId(String),
    Entity(EntityId),
}
impl AnimationTarget {
    pub fn bind_id(&self) -> Option<&str> {
        if let Self::BinderId(id) = self {
            Some(id)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrack {
    pub target: AnimationTarget,
    pub inputs: Vec<f32>, // aka times
    pub outputs: AnimationOutputs,
}
impl AnimationTrack {
    pub fn duration(&self) -> f32 {
        *self.inputs.last().unwrap()
    }
    pub fn break_up_large_rotations(&mut self) {
        let field_values = self.outputs.field_values_mut().unwrap();
        for i in 1..self.inputs.len() {
            let init_value = field_values[i - 1];
            let value_span = field_values[i] - init_value;
            let abs_span = value_span.abs();
            if abs_span >= PI {
                let num_sub_intervals = abs_span / PI;
                let step = value_span / num_sub_intervals;
                let mut next_value = init_value + step;
                let init_time = self.inputs[i - 1];
                let cur_time = self.inputs[i];
                let time_span = cur_time - init_time;
                let interval = time_span / num_sub_intervals;
                let mut next_time = init_time + interval;

                // let mut interpolated_times = Vec::new();
                // let mut interpolated_values = Vec::new();
                let mut p = i;
                while next_time < cur_time {
                    self.inputs.insert(p, next_time);
                    // interpolated_times.push(next_time);
                    next_time += interval;
                    field_values.insert(p, next_value);
                    // inperpolated_values.push(next_value);
                    next_value += step;
                    p += 1;
                }
            }
        }
    }
}

pub struct AnimationTrackInterpolator {
    current_index: usize,
}
impl AnimationTrackInterpolator {
    pub fn new() -> Self {
        Self { current_index: 0 }
    }
    pub fn value(&mut self, track: &AnimationTrack, time: f32) -> AnimationOutput {
        if time < track.inputs[self.current_index] {
            self.current_index = 0;
        }
        while self.current_index + 1 < track.inputs.len()
            && track.inputs[self.current_index + 1] < time
        {
            self.current_index += 1;
        }
        let left = track.outputs.value(self.current_index);
        if self.current_index == track.inputs.len() - 1 {
            return left;
        }
        let input_d = track.inputs[self.current_index + 1] - track.inputs[self.current_index];
        if input_d == 0. {
            return left;
        }
        let right = track.outputs.value(self.current_index + 1);
        let p = (time - track.inputs[self.current_index]) / input_d;
        left.mix(right, p.clamp(0., 1.))
    }
}

pub type AnimationClipFromUrl = BincodeFromUrl<AnimationClip>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AnimationClip {
    pub id: String,
    pub tracks: Vec<AnimationTrack>,
    pub start: f32,
    pub end: f32,
}
impl AnimationClip {
    pub fn from_tracks(tracks: Vec<AnimationTrack>) -> Self {
        let end = tracks
            .iter()
            .map(|x| ordered_float::OrderedFloat::from(x.duration()))
            .max()
            .unwrap()
            .into();
        Self {
            id: "".to_string(),
            tracks,
            start: 0.,
            end,
        }
    }
    pub fn duration(&self) -> f32 {
        self.end - self.start
    }
    /// Merge tracks with Vec3Field outputs into Vec3 and Quat tracks
    pub fn merge_field_tracks(&mut self) {
        let mut euler_rotation_tracks = HashMap::new();
        let mut translation_tracks = HashMap::new();
        let mut scale_tracks = HashMap::new();
        for track in self.tracks.iter() {
            if track.outputs.component() == euler_rotation() {
                let res_tracks = euler_rotation_tracks
                    .entry(track.target.clone())
                    .or_insert_with(HashMap::new);
                res_tracks.insert(track.outputs.field().unwrap(), track.clone());
            } else if track.outputs.component() == translation() {
                let res_tracks = translation_tracks
                    .entry(track.target.clone())
                    .or_insert_with(HashMap::new);
                res_tracks.insert(track.outputs.field().unwrap(), track.clone());
            } else if track.outputs.component() == scale() {
                let res_tracks = scale_tracks
                    .entry(track.target.clone())
                    .or_insert_with(HashMap::new);
                res_tracks.insert(track.outputs.field().unwrap(), track.clone());
            } else {
                panic!("merge_field_tracks is only supported for clips with euler_rotation, translation and scale properties");
            }
        }
        let mut new_tracks = Vec::new();
        for (target, tracks) in euler_rotation_tracks.into_iter() {
            new_tracks.push(merge_rotation_tracks(target, tracks));
        }
        for (target, tracks) in translation_tracks.into_iter() {
            new_tracks.push(merge_vec3_tracks(target, tracks, translation(), Vec3::ZERO));
        }
        for (target, tracks) in scale_tracks.into_iter() {
            new_tracks.push(merge_vec3_tracks(target, tracks, scale(), Vec3::ONE));
        }
        self.tracks = new_tracks;
    }
    pub fn map_outputs(&self, map: impl Fn(&AnimationOutputs) -> AnimationOutputs) -> Self {
        Self {
            tracks: self
                .tracks
                .iter()
                .map(|track| AnimationTrack {
                    outputs: map(&track.outputs),
                    ..(track.clone())
                })
                .collect(),
            ..(self.clone())
        }
    }
}
impl PartialEq for AnimationClip {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
fn merge_track_inputs<'a>(tracks: impl Iterator<Item = &'a AnimationTrack>) -> Vec<f32> {
    let mut inputs = tracks.flat_map(|track| track.inputs.clone()).collect_vec();
    inputs.sort_by_key(|x| OrderedFloat(*x));
    inputs.dedup();
    inputs
}
fn merge_vec3_tracks(
    target: AnimationTarget,
    tracks: HashMap<Vec3Field, AnimationTrack>,
    component: Component<Vec3>,
    default: Vec3,
) -> AnimationTrack {
    let inputs = merge_track_inputs(tracks.values());
    let x = tracks.get(&Vec3Field::X);
    let y = tracks.get(&Vec3Field::Y);
    let z = tracks.get(&Vec3Field::Z);
    let mut val = default;
    let mut outputs = Vec::new();
    for &input in inputs.iter() {
        if let Some(x) = &x {
            val.x = AnimationTrackInterpolator::new()
                .value(x, input)
                .as_field_value()
                .unwrap();
        }
        if let Some(y) = &y {
            val.y = AnimationTrackInterpolator::new()
                .value(y, input)
                .as_field_value()
                .unwrap();
        }
        if let Some(z) = &z {
            val.z = AnimationTrackInterpolator::new()
                .value(z, input)
                .as_field_value()
                .unwrap();
        }
        outputs.push(val);
    }
    AnimationTrack {
        target,
        inputs,
        outputs: AnimationOutputs::Vec3 {
            component,
            data: outputs,
        },
    }
}
fn merge_rotation_tracks(
    target: AnimationTarget,
    tracks: HashMap<Vec3Field, AnimationTrack>,
) -> AnimationTrack {
    let inputs = merge_track_inputs(tracks.values());
    let x = tracks.get(&Vec3Field::X);
    let y = tracks.get(&Vec3Field::Y);
    let z = tracks.get(&Vec3Field::Z);
    let mut euler_rot = Vec3::ZERO;
    let mut outputs = Vec::new();
    for &input in inputs.iter() {
        if let Some(x) = &x {
            euler_rot.x = AnimationTrackInterpolator::new()
                .value(x, input)
                .as_field_value()
                .unwrap();
        }
        if let Some(y) = &y {
            euler_rot.y = AnimationTrackInterpolator::new()
                .value(y, input)
                .as_field_value()
                .unwrap();
        }
        if let Some(z) = &z {
            euler_rot.z = AnimationTrackInterpolator::new()
                .value(z, input)
                .as_field_value()
                .unwrap();
        }
        let quat = Quat::from_euler(EulerRot::ZYX, euler_rot.z, euler_rot.y, euler_rot.x);
        outputs.push(quat);
    }
    AnimationTrack {
        target,
        inputs,
        outputs: AnimationOutputs::Quat {
            component: rotation(),
            data: outputs,
        },
    }
}
