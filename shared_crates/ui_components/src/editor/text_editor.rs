use std::{self, str::FromStr};

use ambient_element::{element_component, Element, ElementComponentExt, Hooks};

use glam::*;

use crate::{layout::FlowRow, text::Text, use_focus, Rectangle, UIBase, UIExt};
use ambient_cb::{cb, Cb};
use ambient_guest_bridge::{
    components::{
        input::{event_keyboard_input, event_received_character, keycode},
        layout::{height, min_height, min_width, width},
        rendering::color,
        text::text,
        transform::translation,
    },
    window::set_cursor,
};
use ambient_shared_types::events::{WINDOW_KEYBOARD_INPUT, WINDOW_RECEIVED_CHARACTER};
use ambient_window_types::{CursorIcon, VirtualKeyCode};

use super::{Editor, EditorOpts};
#[cfg(feature = "native")]
use ambient_sys::time::Instant;
use itertools::Itertools;
#[cfg(feature = "guest")]
use std::time::Instant;

#[element_component]
pub fn TextEditor(
    hooks: &mut Hooks,
    value: String,
    on_change: Cb<dyn Fn(String) + Sync + Send>,
    on_submit: Option<Cb<dyn Fn(String) + Sync + Send>>,
    password: bool,
    placeholder: Option<String>,
) -> Element {
    let (focused, set_focused) = use_focus(hooks);
    let (command, set_command) = hooks.use_state(false);
    let intermediate_value = hooks.use_ref_with(|_| value.clone());
    let cursor_position = hooks.use_ref_with(|_| value.len());
    let rerender = hooks.use_rerender_signal();
    {
        let mut inter = intermediate_value.lock();
        if *inter != value {
            let mut cp = cursor_position.lock();
            *cp = cp.min(value.len());
        }
        *inter = value.clone();
    }

    hooks.use_spawn({
        let set_focused = set_focused.clone();
        move |_| {
            Box::new(move |_| {
                if focused {
                    set_focused(false);
                }
            })
        }
    });
    hooks.use_multi_event(&[WINDOW_RECEIVED_CHARACTER, WINDOW_KEYBOARD_INPUT], {
        let value = intermediate_value.clone();
        let on_change = on_change.clone();
        let cursor_position = cursor_position.clone();
        move |_world, event| {
            if let Some(c) = event.get_ref(event_received_character()) {
                let c = c.chars().next().unwrap();
                if command || !focused {
                    return;
                }
                if c == '\u{7f}' || c == '\u{8}' {
                    if *cursor_position.lock() > 0 {
                        let mut value = value.lock();
                        value.remove(*cursor_position.lock() - 1);
                        *cursor_position.lock() -= 1;
                        on_change.0(value.clone());
                    }
                } else if c == '\r' {
                    if let Some(on_submit) = on_submit.clone() {
                        on_submit.0(value.lock().clone());
                    }
                } else if c != '\t' && c != '\n' && c != '\r' {
                    let mut value = value.lock();
                    value.insert(*cursor_position.lock(), c);
                    *cursor_position.lock() += 1;
                    on_change.0(value.clone());
                }
            } else if let Some(pressed) = event.get(event_keyboard_input()) {
                if !focused {
                    return;
                }
                if let Some(kc) = event.get_ref(keycode()) {
                    // FIXME: get_ref returns `&T` on native, but `T` on guest
                    let kc = VirtualKeyCode::from_str(&kc).unwrap();
                    match kc {
                        VirtualKeyCode::LWin => {
                            #[cfg(target_os = "macos")]
                            set_command(pressed);
                        }
                        VirtualKeyCode::LControl => {
                            #[cfg(not(target_os = "macos"))]
                            set_command(pressed);
                        }
                        VirtualKeyCode::V => {
                            if command && pressed {
                                #[cfg(not(target_os = "unknown"))]
                                if let Some(paste) = ambient_guest_bridge::window::get_clipboard() {
                                    let mut value = value.lock();
                                    value.insert_str(*cursor_position.lock(), &paste);
                                    *cursor_position.lock() += paste.len();
                                    on_change.0(value.clone());
                                }
                            }
                        }
                        VirtualKeyCode::Left => {
                            if pressed && *cursor_position.lock() > 0 {
                                *cursor_position.lock() -= 1;
                                rerender();
                            }
                        }
                        VirtualKeyCode::Right => {
                            if pressed && *cursor_position.lock() < value.lock().len() {
                                *cursor_position.lock() += 1;
                                rerender();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });
    let (a, b) = value.split_at(*cursor_position.lock());
    let [a, b]: [Element; 2] = [a, b]
        .iter()
        .map(|value| {
            Text.el()
                .with(text(), if password { value.chars().map(|_| '*').collect() } else { value.to_string() })
                .with(color(), vec4(0.9, 0.9, 0.9, 1.))
        })
        .collect_vec()
        .try_into()
        .unwrap();

    if focused {
        FlowRow::el([a, Cursor.el(), b])
    } else if value.is_empty() && !focused && placeholder.is_some() {
        Text.el().with(text(), placeholder.clone().unwrap()).with(color(), vec4(1., 1., 1., 0.2))
    } else {
        FlowRow::el([a, b])
    }
    .with(min_width(), 3.)
    .with(min_height(), 13.)
    .with_clickarea()
    .on_mouse_up(move |_, _, _| {
        set_focused(true);
    })
    .on_mouse_enter(|world, _| {
        set_cursor(world, CursorIcon::Text);
    })
    .on_mouse_leave(|world, _| {
        set_cursor(world, CursorIcon::Default);
    })
    .el()
}

impl TextEditor {
    pub fn new(value: String, on_change: Cb<dyn Fn(String) + Sync + Send>) -> Self {
        Self { value, on_change, on_submit: None, password: false, placeholder: None }
    }
    pub fn on_submit(mut self, on_submit: impl Fn(String) + Sync + Send + 'static) -> Self {
        self.on_submit = Some(cb(on_submit));
        self
    }
    pub fn placeholder<T: Into<String>>(mut self, placeholder: Option<T>) -> Self {
        self.placeholder = placeholder.map(|x| x.into());
        self
    }
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
}

#[element_component]
fn Cursor(_hooks: &mut Hooks) -> Element {
    CursorInner::el(Instant::now())
}
#[element_component]
fn CursorInner(hooks: &mut Hooks, render_time: Instant) -> Element {
    let rerender = hooks.use_rerender_signal();
    hooks.use_frame(move |_| rerender());
    let delta = (Instant::now().duration_since(render_time).as_secs_f32() * 2.) as u32;
    if delta % 2 == 0 {
        UIBase.el().children(vec![Rectangle.el().with(width(), 2.).with(height(), 13.).with(translation(), vec3(1., 0., 0.))])
    } else {
        Element::new()
    }
}

impl Editor for String {
    fn editor(self, on_change: Cb<dyn Fn(Self) + Sync + Send>, _: EditorOpts) -> Element {
        TextEditor::new(self, on_change).placeholder(Some("Empty")).el()
    }

    fn view(self, _opts: EditorOpts) -> Element {
        Text.el().with(text(), self)
    }
}
