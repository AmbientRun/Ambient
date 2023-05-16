use std::collections::HashMap;

use ambient_animation::{
    animation_bind_id_from_name, AnimationClip, AnimationOutputs, AnimationTarget, AnimationTrack,
    Vec3Field,
};
use ambient_core::transform::{euler_rotation, scale, translation};
use fbxcel::tree::v7400::NodeHandle;
use itertools::Itertools;
use ordered_float::OrderedFloat;

use super::FbxDoc;

// From: https://help.autodesk.com/view/FBX/2015/ENU/?guid=__cpp_ref_class_fbx_anim_curve_html
const FBX_TIME: f32 = 46186158000.;

pub fn get_animations(doc: &FbxDoc) -> HashMap<String, AnimationClip> {
    doc.animation_stacks
        .values()
        .map(|stack| {
            let mut clip = AnimationClip {
                id: stack.name.clone(),
                tracks: stack
                    .layers
                    .iter()
                    .flat_map(|layer_id| {
                        let layer = doc.animation_layers.get(layer_id).unwrap();
                        layer.curve_nodes.iter().flat_map(|curve_node_id| {
                            let curve_node = doc.animation_curve_nodes.get(curve_node_id).unwrap();
                            curve_node.outputs.iter().flat_map(|(output_id, property)| {
                                curve_node
                                    .curves
                                    .iter()
                                    .map(|(field, curve_id)| {
                                        let curve = doc.animation_curves.get(curve_id).unwrap();
                                        let node = doc.models.get(output_id).unwrap();
                                        let track = AnimationTrack {
                                            target: AnimationTarget::BinderId(
                                                animation_bind_id_from_name(&node.node_name),
                                            ),
                                            inputs: curve
                                                .key_time
                                                .iter()
                                                .map(|time| *time as f32 / FBX_TIME)
                                                .collect(), // TODO
                                            outputs: AnimationOutputs::Vec3Field {
                                                component: match property as &str {
                                                    "Lcl Translation" => translation(),
                                                    "Lcl Scaling" => scale(),
                                                    "Lcl Rotation" => euler_rotation(),
                                                    _ => panic!(
                                                        "Unsupported target property: {property}"
                                                    ),
                                                },
                                                field: match field as &str {
                                                    "d|X" => Vec3Field::X,
                                                    "d|Y" => Vec3Field::Y,
                                                    "d|Z" => Vec3Field::Z,
                                                    _ => panic!("Unsupported field: {field}"),
                                                },
                                                data: match property as &str {
                                                    "Lcl Rotation" => curve
                                                        .key_value_float
                                                        .iter()
                                                        .map(|v| v.to_radians())
                                                        .collect(),
                                                    _ => curve.key_value_float.clone(),
                                                },
                                            },
                                        };
                                        // if property == "Lcl Rotation" {
                                        //     track.break_up_large_rotations();
                                        // }
                                        track
                                    })
                                    .collect_vec()
                                    .into_iter()
                            })
                        })
                    })
                    .collect(),

                start: stack.local_start.map(|x| x as f32 / FBX_TIME).unwrap_or(0.),
                end: match stack.local_stop {
                    Some(x) => x as f32 / FBX_TIME,
                    None => {
                        let max_time = stack
                            .layers
                            .iter()
                            .flat_map(|layer_id| {
                                let layer = doc.animation_layers.get(layer_id).unwrap();
                                layer
                                    .curve_nodes
                                    .iter()
                                    .flat_map(|curve_node_id| {
                                        let curve_node =
                                            doc.animation_curve_nodes.get(curve_node_id).unwrap();
                                        curve_node
                                            .curves
                                            .iter()
                                            .flat_map(|(_field, curve_id)| {
                                                let curve =
                                                    doc.animation_curves.get(curve_id).unwrap();
                                                curve
                                                    .key_time
                                                    .last()
                                                    .map(|x| OrderedFloat(*x as f32 / FBX_TIME))
                                            })
                                            .max()
                                    })
                                    .max()
                            })
                            .max()
                            .map(|x| x.0)
                            .unwrap_or(0.);
                        max_time
                    }
                },
            };
            clip.merge_field_tracks();
            (stack.name.clone(), clip)
        })
        .collect()
}

#[derive(Debug)]
pub struct FbxAnimationStack {
    pub id: i64,
    pub name: String,
    pub local_start: Option<i64>,
    pub local_stop: Option<i64>,
    pub layers: Vec<i64>,
}
impl FbxAnimationStack {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let name = node.attributes()[1]
            .get_string()
            .unwrap()
            .split('\u{0}')
            .next()
            .unwrap();
        let name = if name.contains('|') {
            name.split('|').nth(1).unwrap().to_string()
        } else {
            name.to_string()
        };
        let props = node.children().find(|node| node.name() == "Properties70");
        let mut local_stop = None;
        let mut local_start = None;
        if let Some(props) = props {
            if let Some(local_stop_node) = props
                .children()
                .find(|node| node.attributes()[0].get_string().unwrap() == "LocalStop")
            {
                local_stop = Some(local_stop_node.attributes()[4].get_i64().unwrap());
            }
            if let Some(local_start_node) = props
                .children()
                .find(|node| node.attributes()[0].get_string().unwrap() == "LocalStart")
            {
                local_start = Some(local_start_node.attributes()[4].get_i64().unwrap());
            }
        }
        Self {
            id,
            name,
            local_stop,
            local_start,
            layers: Vec::new(),
        }
    }
}
#[derive(Debug)]
pub struct FbxAnimationLayer {
    pub curve_nodes: Vec<i64>,
}
#[derive(Debug)]
pub struct FbxAnimationCurveNode {
    pub id: i64,
    pub curves: HashMap<String, i64>,
    pub outputs: Vec<(i64, String)>,
}
impl FbxAnimationCurveNode {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        Self {
            id,
            curves: HashMap::new(),
            outputs: Vec::new(),
        }
    }
}
#[derive(Debug)]
pub struct FbxAnimationCurve {
    pub id: i64,
    pub key_time: Vec<i64>,
    pub key_value_float: Vec<f32>,
}
impl FbxAnimationCurve {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let key_time = node
            .children()
            .find(|node| node.name() == "KeyTime")
            .unwrap();
        let key_value_float = node
            .children()
            .find(|node| node.name() == "KeyValueFloat")
            .unwrap();
        Self {
            id,
            key_time: key_time.attributes()[0].get_arr_i64().unwrap().to_vec(),
            key_value_float: key_value_float.attributes()[0]
                .get_arr_f32()
                .unwrap()
                .to_vec(),
        }
    }
}
