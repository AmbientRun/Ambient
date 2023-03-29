use std::{self, str::FromStr, time::Duration};

use ambient_element::{element_component, Element, ElementComponentExt, Hooks};

use glam::*;

use crate::{text::Text, use_focus, Rectangle, UIBase, UIExt};
use ambient_cb::{cb, Cb};
use ambient_event_types::{WINDOW_KEYBOARD_INPUT, WINDOW_RECEIVED_CHARACTER};
use ambient_guest_bridge::{
    components::{
        input::{event_keyboard_input, event_received_character, keycode},
        layout::{align_horizontal_end, fit_horizontal_none, fit_vertical_none, height, layout_flow, min_height, min_width, width},
        rendering::color,
        text::text,
        transform::translation,
    },
    window::set_cursor,
};
use ambient_window_types::{CursorIcon, VirtualKeyCode};

use super::{Editor, EditorOpts};

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
    *intermediate_value.lock() = value.clone();
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
        move |_world, event| {
            if let Some(c) = event.get_ref(event_received_character()) {
                let c = c.chars().next().unwrap();
                if command || !focused {
                    return;
                }
                if c == '\u{7f}' || c == '\u{8}' {
                    let mut value = value.lock();
                    value.pop();
                    on_change.0(value.clone());
                } else if c == '\r' {
                    if let Some(on_submit) = on_submit.clone() {
                        on_submit.0(value.lock().clone());
                    }
                } else if c != '\t' && c != '\n' && c != '\r' {
                    let mut value = value.lock();
                    *value = format!("{value}{c}");
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
                                    *value = format!("{value}{paste}");
                                    on_change.0(value.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });
    let el = if value.is_empty() && !focused && placeholder.is_some() {
        Text.el().with(text(), placeholder.unwrap()).with(color(), vec4(1., 1., 1., 0.2))
    } else {
        Text.el().with(text(), if password { value.chars().map(|_| '*').collect() } else { value }).with(color(), vec4(0.9, 0.9, 0.9, 1.))
    }
    .init_default(layout_flow())
    .with_default(fit_horizontal_none())
    .with_default(fit_vertical_none())
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
    .el();

    if focused {
        el.with_default(align_horizontal_end()).children(vec![Cursor.el()])
    } else {
        el
    }
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
pub fn Cursor(hooks: &mut Hooks) -> Element {
    let (show, set_show) = hooks.use_state(true);
    hooks.use_interval_deps(Duration::from_millis(500), false, show, move |show| set_show(!show));
    if show {
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
