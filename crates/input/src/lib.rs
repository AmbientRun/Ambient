use std::{cmp::Reverse, collections::HashSet};

use ambient_ecs::{components, query, world_events, EntityData, EntityId, QueryState, System, SystemGroup, World};
use ambient_std::events::EventDispatcher;
use glam::{vec2, Vec2};
use serde::{Deserialize, Serialize};
pub use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use winit::event::{ModifiersState, ScanCode};

use crate::picking::picking_winit_event_system;

pub mod picking;

pub type EventCallback<Event, Ret = bool> = EventDispatcher<dyn Fn(&mut World, EntityId, Event) -> Ret + Sync + Send>;

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
    on_app_mouse_motion: EventCallback<Vec2, ()>,
    on_app_mouse_wheel: EventCallback<MouseScrollDelta>,
    on_app_modifiers_change: EventCallback<ModifiersState, ()>,
    on_app_focus_change: EventCallback<bool, ()>,

    player_raw_input: PlayerRawInput,
    player_prev_raw_input: PlayerRawInput,
});

pub fn init_all_components() {
    picking::init_components();
    init_components();
}

pub fn event_systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new("inputs", vec![Box::new(InputSystem::new()), Box::new(picking_winit_event_system())])
}

#[derive(Debug)]
pub struct InputSystem {
    modifiers: ModifiersState,
    is_focused: bool,
    keyboard_event_qs: QueryState,
    mouse_input_qs: QueryState,
    mouse_wheel_qs: QueryState,
    mouse_motion_qs: QueryState,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            keyboard_event_qs: QueryState::new(),
            mouse_input_qs: QueryState::new(),
            mouse_wheel_qs: QueryState::new(),
            mouse_motion_qs: QueryState::new(),
            modifiers: ModifiersState::empty(),
            is_focused: true,
        }
    }
}

impl System<Event<'static, ()>> for InputSystem {
    fn run(&mut self, world: &mut ambient_ecs::World, event: &Event<'static, ()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                &WindowEvent::Focused(focused) => {
                    self.is_focused = focused;
                    let mut fire_event = |world: &mut World| {
                        let mut handlers = query((on_app_focus_change(),)).collect_cloned(world, Some(&mut self.keyboard_event_qs));
                        handlers.sort_by_key(|(_, (handler,))| Reverse(handler.created_timestamp));
                        for (id, (dispatcher,)) in handlers {
                            for handler in dispatcher.iter() {
                                handler(world, id, focused)
                            }
                        }
                    };
                    fire_event(world);
                }
                WindowEvent::ReceivedCharacter(c) => {
                    world.resource_mut(world_events()).add_event(EntityData::new().set(event_received_character(), *c));
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    let event = KeyboardEvent {
                        scancode: input.scancode,
                        state: input.state,
                        keycode: input.virtual_keycode,
                        modifiers: self.modifiers,
                        is_focused: self.is_focused,
                    };
                    world.resource_mut(world_events()).add_event(EntityData::new().set(event_keyboard_input(), event));
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    world
                        .resource_mut(world_events())
                        .add_event(EntityData::new().set(event_mouse_input(), MouseInput { state: *state, button: *button }));
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    let mut fire_wheel_event = |world: &mut World| {
                        let mut handlers = query((on_app_mouse_wheel(),)).collect_cloned(world, Some(&mut self.mouse_wheel_qs));
                        handlers.sort_by_key(|(_, (handler,))| Reverse(handler.created_timestamp));
                        for (id, (dispatcher,)) in handlers {
                            for handler in dispatcher.iter() {
                                if handler(world, id, *delta) {
                                    return;
                                }
                            }
                        }
                    };
                    fire_wheel_event(world);
                }
                WindowEvent::ModifiersChanged(mods) => {
                    self.modifiers = *mods;
                    let mut fire_event = |world: &mut World| {
                        let mut handlers = query((on_app_modifiers_change(),)).collect_cloned(world, Some(&mut self.keyboard_event_qs));
                        handlers.sort_by_key(|(_, (handler,))| Reverse(handler.created_timestamp));
                        for (id, (dispatcher,)) in handlers {
                            for handler in dispatcher.iter() {
                                handler(world, id, *mods)
                            }
                        }
                    };
                    fire_event(world);
                }

                _ => {}
            },

            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                for (id, (dispatcher,)) in query((on_app_mouse_motion(),)).collect_cloned(world, Some(&mut self.mouse_motion_qs)) {
                    for handle in dispatcher.iter() {
                        handle(world, id, vec2(delta.0 as f32, delta.1 as f32));
                    }
                }
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
