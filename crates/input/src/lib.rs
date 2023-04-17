use std::collections::HashSet;

use ambient_ecs::{components, generated::messages, world_events, Debuggable, Entity, Resource, System, SystemGroup, WorldEventsExt};
use glam::{vec2, Vec2};
use serde::{Deserialize, Serialize};
use winit::event::ModifiersState;
pub use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};

pub mod picking;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PlayerRawInput {
    pub keys: HashSet<ambient_window_types::VirtualKeyCode>,
    pub mouse_position: Vec2,
    /// cursor position is _not_ the sum of mouse_deltas; mouse_delta is
    pub cursor_position: Vec2,
    pub mouse_wheel: f32,
    pub mouse_buttons: HashSet<ambient_window_types::MouseButton>,
}

components!("input", {
    event_modifiers_change: ModifiersState,

    @[Debuggable, Resource]
    player_raw_input: PlayerRawInput,
    @[Debuggable, Resource]
    player_prev_raw_input: PlayerRawInput,
});

pub fn init_all_components() {
    picking::init_components();
    init_components();
}

pub fn event_systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new("inputs", vec![Box::new(InputSystem::new())])
}

pub fn resources() -> Entity {
    Entity::new().with_default(player_raw_input()).with_default(player_prev_raw_input())
}

#[derive(Debug)]
pub struct InputSystem {
    modifiers: ModifiersState,
    is_focused: bool,
}

impl InputSystem {
    pub fn new() -> Self {
        Self { modifiers: ModifiersState::empty(), is_focused: true }
    }
}

impl System<Event<'static, ()>> for InputSystem {
    fn run(&mut self, world: &mut ambient_ecs::World, event: &Event<'static, ()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                &WindowEvent::Focused(focused) => {
                    self.is_focused = focused;
                    world.resource_mut(world_events()).add_message(messages::WindowFocusChange::new(focused));
                }
                WindowEvent::ReceivedCharacter(c) => {
                    world.resource_mut(world_events()).add_message(messages::WindowKeyboardCharacter::new(c.to_string()));
                }

                WindowEvent::ModifiersChanged(mods) => {
                    self.modifiers = *mods;
                    world.resource_mut(world_events()).add_message(messages::WindowKeyboardModifiersChange::new(mods.bits()));
                }

                WindowEvent::CloseRequested => {
                    world.resource_mut(world_events()).add_message(messages::WindowClose::new());
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    let keycode = input.virtual_keycode.map(|key| ambient_window_types::VirtualKeyCode::from(key).to_string());
                    let modifiers = self.modifiers.bits();
                    let pressed = match input.state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                    world.resource_mut(world_events()).add_message(messages::WindowKeyboardInput::new(keycode, modifiers, pressed));
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    world.resource_mut(world_events()).add_message(messages::WindowMouseInput::new(
                        ambient_window_types::MouseButton::from(*button),
                        match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        },
                    ));
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    world.resource_mut(world_events()).add_message(messages::WindowMouseWheel::new(
                        match *delta {
                            MouseScrollDelta::LineDelta(x, y) => vec2(x, y),
                            MouseScrollDelta::PixelDelta(p) => vec2(p.x as f32, p.y as f32),
                        },
                        matches!(delta, MouseScrollDelta::PixelDelta(..)),
                    ));
                }

                _ => {}
            },

            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                world.resource_mut(world_events()).add_message(messages::WindowMouseMotion::new(vec2(delta.0 as f32, delta.1 as f32)));
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct MouseInput {
    pub state: ElementState,
    pub button: MouseButton,
}
