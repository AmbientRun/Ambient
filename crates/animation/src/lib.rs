use ambient_ecs::SystemGroup;
use player::animation_player_systems;

mod player;
mod resources;
mod retargeting;

pub use resources::*;
pub use retargeting::*;

pub fn init_all_components() {
    player::init_components();
}

pub fn animation_systems() -> SystemGroup {
    animation_player_systems()
}

#[test]
fn test_animation() {
    use ambient_core::transform::{self, translation};
    use glam::vec3;

    transform::init_components();
    ambient_ecs::init_components();

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
