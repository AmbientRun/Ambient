//! Defines the [DragArea] element.

use ambient_element::{to_owned, Element, ElementComponent, Hooks};
use ambient_guest_bridge::components::transform::{local_to_parent, local_to_world};
use ambient_guest_bridge::components::{app::cursor_position, transform::translation};
use ambient_guest_bridge::{
    components::input::{mouse_over, mouse_pickable_max, mouse_pickable_min},
    messages,
};
use ambient_shared_types::CursorIcon;
use glam::Vec3;

#[derive(Debug, Clone)]
/// An area that can be dragged.
pub struct DragArea {
    /// The inner element.
    pub inner: Element,
}

impl DragArea {
    /// Create a new ClickArea.
    pub fn new(inner: Element) -> Self {
        Self { inner }
    }
}
impl ElementComponent for DragArea {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { inner } = *self;
        let id = hooks.use_ref_with(|_| None);
        let mouse_over_count = hooks.use_ref_with(|_| 0);
        let (moving, set_moving) = hooks.use_state(false);
        let (mouse_click_pos, set_mouse_click_pos) = hooks.use_state((0.0, 0.0));
        let (click_pos, set_click_pos) = hooks.use_state((0.0, 0.0));
        hooks.use_frame({
            to_owned![id, mouse_over_count];
            move |world| {
                if let Some(id) = *id.lock() {
                    if moving {
                        let mouse_pos = world.resource(cursor_position());
                        let pos = world.get(id, translation()).unwrap_or(Vec3::ZERO);
                        world
                            .set(
                                id,
                                translation(),
                                Vec3 {
                                    x: click_pos.0 + mouse_pos.x - mouse_click_pos.0,
                                    y: click_pos.1 + mouse_pos.y - mouse_click_pos.1,
                                    z: pos.z,
                                },
                            )
                            .unwrap();
                    }
                    let next = world.get(id, mouse_over()).unwrap_or(0);
                    let mut state = mouse_over_count.lock();
                    // if *state == 0 && next > 0 {
                    //     // println!("mouse enter");
                    //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Move);
                    // }
                    // if *state > 0 && next == 0 {
                    //     // println!("mouse leave");
                    //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Default);
                    // }

                    if next > 0 {
                        ambient_guest_bridge::window::set_cursor(world, CursorIcon::Move);
                    } else {
                        ambient_guest_bridge::window::set_cursor(world, CursorIcon::Arrow);
                    }

                    *state = next;
                }
            }
        });

        hooks.use_runtime_message::<messages::WindowMouseInput>({
            to_owned![id, mouse_over_count, set_moving];
            move |world, event| {
                if let Some(id) = *id.lock() {
                    if *mouse_over_count.lock() > 0 {
                        if event.pressed && event.button == 0 {
                            set_moving(true);
                            let mouse_pos = world.resource(cursor_position());
                            let pos = world.get(id, translation()).unwrap_or(Vec3::ZERO);
                            set_mouse_click_pos((mouse_pos.x, mouse_pos.y));
                            set_click_pos((pos.x, pos.y));
                        } else {
                            set_moving(false);
                        }
                    } else {
                        set_moving(false);
                    }
                }
            }
        });

        inner
            .init(mouse_pickable_min(), Vec3::ZERO)
            .init(mouse_pickable_max(), Vec3::ZERO)
            .on_spawned(move |_, new_id, _| {
                *id.lock() = Some(new_id);
            })
    }
}
