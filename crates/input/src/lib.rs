use std::collections::HashSet;

use ambient_ecs::{components, world_events, Debuggable, Entity, Resource, System, SystemGroup};
use ambient_shared_types::events;
use glam::{vec2, Vec2};
use serde::{Deserialize, Serialize};
use winit::event::ModifiersState;
pub use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};

pub use ambient_ecs::generated::components::core::input::{
    event_focus_change, event_keyboard_input, event_mouse_input, event_mouse_motion, event_mouse_wheel, event_mouse_wheel_pixels,
    event_received_character, keyboard_modifiers, keycode, mouse_button,
};

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
                    world
                        .resource_mut(world_events())
                        .add_event((events::WINDOW_FOCUSED.to_string(), Entity::new().with(event_focus_change(), focused)));
                }
                WindowEvent::ReceivedCharacter(c) => {
                    world.resource_mut(world_events()).add_event((
                        events::WINDOW_RECEIVED_CHARACTER.to_string(),
                        Entity::new().with(event_received_character(), c.to_string()),
                    ));
                }

                WindowEvent::ModifiersChanged(mods) => {
                    self.modifiers = *mods;
                    world
                        .resource_mut(world_events())
                        .add_event((events::WINDOW_MODIFIERS_CHANGED.to_string(), Entity::new().with(event_modifiers_change(), *mods)));
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    let mut data = Entity::new()
                        .with(
                            event_keyboard_input(),
                            match input.state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            },
                        )
                        .with(keyboard_modifiers(), self.modifiers.bits());
                    if let Some(key) = input.virtual_keycode {
                        data.set(keycode(), ambient_window_types::VirtualKeyCode::from(key).to_string());
                    }
                    world.resource_mut(world_events()).add_event((events::WINDOW_KEYBOARD_INPUT.to_string(), data));
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    world.resource_mut(world_events()).add_event((
                        events::WINDOW_MOUSE_INPUT.to_string(),
                        Entity::new()
                            .with(
                                event_mouse_input(),
                                match state {
                                    ElementState::Pressed => true,
                                    ElementState::Released => false,
                                },
                            )
                            .with(mouse_button(), ambient_window_types::MouseButton::from(*button).into()),
                    ));
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    world.resource_mut(world_events()).add_event((
                        events::WINDOW_MOUSE_WHEEL.to_string(),
                        Entity::new()
                            .with(
                                event_mouse_wheel(),
                                match *delta {
                                    MouseScrollDelta::LineDelta(x, y) => vec2(x, y),
                                    MouseScrollDelta::PixelDelta(p) => vec2(p.x as f32, p.y as f32),
                                },
                            )
                            .with(event_mouse_wheel_pixels(), matches!(delta, MouseScrollDelta::PixelDelta(..))),
                    ));
                }

                _ => {}
            },

            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                world.resource_mut(world_events()).add_event((
                    events::WINDOW_MOUSE_MOTION.to_string(),
                    Entity::new().with(event_mouse_motion(), vec2(delta.0 as f32, delta.1 as f32)),
                ));
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
