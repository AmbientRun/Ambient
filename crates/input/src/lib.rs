use std::collections::HashSet;

use ambient_ecs::{components, world_events, Entity, System, SystemGroup};
use glam::{vec2, Vec2};
use serde::{Deserialize, Serialize};
pub use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use winit::event::{ModifiersState, ScanCode};

pub mod picking;

#[derive(Debug, Clone)]
/// Represents a keyboard event with attached modifiers
pub struct KeyboardEvent {
    pub scancode: ScanCode,
    pub state: ElementState,
    pub keycode: Option<VirtualKeyCode>,
    pub modifiers: ModifiersState,
    pub is_focused: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PlayerRawInput {
    pub keys: HashSet<VirtualKeyCode>,
    pub mouse_position: Vec2,
    pub mouse_wheel: f32,
    pub mouse_buttons: HashSet<MouseButton>,
}

components!("input", {
    event_received_character: char,
    event_keyboard_input: KeyboardEvent,
    event_mouse_input: MouseInput,
    event_mouse_motion: Vec2,
    event_mouse_wheel: MouseScrollDelta,
    event_modifiers_change: ModifiersState,
    event_focus_change: bool,

    player_raw_input: PlayerRawInput,
    player_prev_raw_input: PlayerRawInput,
});

pub fn init_all_components() {
    picking::init_components();
    init_components();
}

pub fn event_systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new("inputs", vec![Box::new(InputSystem::new())])
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
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_focus_change(), focused));
                }
                WindowEvent::ReceivedCharacter(c) => {
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_received_character(), *c));
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    let event = KeyboardEvent {
                        scancode: input.scancode,
                        state: input.state,
                        keycode: input.virtual_keycode,
                        modifiers: self.modifiers,
                        is_focused: self.is_focused,
                    };
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_keyboard_input(), event));
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    world
                        .resource_mut(world_events())
                        .add_event(Entity::new().with(event_mouse_input(), MouseInput { state: *state, button: *button }));
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_mouse_wheel(), *delta));
                }
                WindowEvent::ModifiersChanged(mods) => {
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_modifiers_change(), *mods));
                }

                _ => {}
            },

            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                world
                    .resource_mut(world_events())
                    .add_event(Entity::new().with(event_mouse_motion(), vec2(delta.0 as f32, delta.1 as f32)));
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
