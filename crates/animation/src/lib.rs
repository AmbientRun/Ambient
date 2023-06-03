use ambient_ecs::{components, Debuggable, Networked, Store, SystemGroup};
use convert_case::{Case, Casing};
use graph::animation_graph_systems;

mod graph;
mod resources;
mod retargeting;

pub use resources::*;
pub use retargeting::*;

components!("animation", {
    @[Debuggable, Networked, Store]
    animation_errors: String,
});

pub fn init_all_components() {
    init_components();
    graph::init_components();
}

pub fn animation_systems() -> SystemGroup {
    SystemGroup::new(
        "animation_systems",
        vec![Box::new(animation_graph_systems())],
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
