use std::collections::HashSet;

use elements_ecs::components;
use elements_input::{MouseButton, VirtualKeyCode};
use glam::Vec2;
use serde::{Deserialize, Serialize};

components!("player", {
    raw_input: RawInput,
    prev_raw_input: RawInput,
});

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct RawInput {
    pub keys: HashSet<VirtualKeyCode>,
    pub mouse_position: Vec2,
    pub mouse_wheel: f32,
    pub mouse_buttons: HashSet<MouseButton>,
}
