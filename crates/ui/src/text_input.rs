use std::{self, time::Duration};

use ambient_core::{transform::translation, window::window_ctl, window::WindowCtl};
use ambient_ecs::EntityId;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_input::{event_keyboard_input, event_received_character, keycode};
use ambient_renderer::color;
use ambient_std::{cb, Cb};
use closure::closure;
use glam::*;
use winit::{event::VirtualKeyCode, window::CursorIcon};

use super::{Editor, EditorOpts, Focus, Text, UIExt};
use crate::{layout::*, text, use_interval_deps, Rectangle, UIBase};
use ambient_event_types::{WINDOW_KEYBOARD_INPUT, WINDOW_RECEIVED_CHARACTER};

#[element_component]
pub fn TextInput(
    hooks: &mut Hooks,
    value: String,
    on_change: Cb<dyn Fn(String) + Sync + Send>,
    on_submit: Option<Cb<dyn Fn(String) + Sync + Send>>,
    password: bool,
    placeholder: Option<String>,
) -> Element {
    let (self_id, set_self_id) = hooks.use_state(EntityId::null());
    let (focus, set_focus) = hooks.consume_context::<Focus>().expect("No FocusRoot available");
    let focused = focus == Focus(Some(self_id));
    let (command, set_command) = hooks.use_state(false);
    hooks.use_spawn(closure!(clone set_focus, |_| {
        Box::new(move |_| {
            if focused {
                set_focus(Focus(None));
            }
        })
    }));
    hooks.use_multi_event(&[WINDOW_RECEIVED_CHARACTER, WINDOW_KEYBOARD_INPUT], {
        let value = value.clone();
        let on_change = on_change.clone();
        move |_world, event| {
            if let Some(c) = event.get(event_received_character()) {
                if command || !focused {
                    return;
                }
                if c == '\u{7f}' || c == '\u{8}' {
                    let mut value = value.clone();
                    value.pop();
                    on_change.0(value);
                } else if c == '\r' {
                    if let Some(on_submit) = on_submit.clone() {
                        on_submit.0(value.clone());
                    }
                } else if c != '\t' && c != '\n' && c != '\r' {
                    on_change.0(format!("{value}{c}"))
                }
            } else if let Some(pressed) = event.get(event_keyboard_input()) {
                if !focused {
                    return;
                }
                if let Some(kc) = event.get_ref(keycode()) {
                    let kc: VirtualKeyCode = serde_json::from_str(kc).unwrap();
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
                                if let Ok(paste) = arboard::Clipboard::new().unwrap().get_text() {
                                    on_change.0(format!("{value}{paste}"));
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
        Text.el().set(text(), placeholder.unwrap()).set(color(), vec4(1., 1., 1., 0.2))
    } else {
        Text.el().set(text(), if password { value.chars().map(|_| '*').collect() } else { value }).set(color(), vec4(0.9, 0.9, 0.9, 1.))
    }
    .init(layout(), Layout::Flow)
    .set(fit_horizontal(), Fit::None)
    .set(fit_vertical(), Fit::None)
    .set(min_width(), 3.)
    .set(min_height(), 13.)
    .on_spawned(move |_, id| set_self_id(id))
    .with_clickarea()
    .on_mouse_up(move |_, id, _| {
        set_focus(Focus(Some(id)));
    })
    .on_mouse_enter(|world, _| {
        world.resource(window_ctl()).send(WindowCtl::SetCursorIcon(CursorIcon::Text)).ok();
    })
    .on_mouse_leave(|world, _| {
        world.resource(window_ctl()).send(WindowCtl::SetCursorIcon(CursorIcon::Default)).ok();
    })
    .el();

    if focused {
        el.set(align_horizontal(), Align::End).children(vec![Cursor.el()])
    } else {
        el
    }
}

impl TextInput {
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

impl Editor for String {
    fn editor(self, on_change: Cb<dyn Fn(Self) + Sync + Send>, _: EditorOpts) -> Element {
        TextInput::new(self, on_change).placeholder(Some("Empty")).el()
    }

    fn view(self, _opts: EditorOpts) -> Element {
        Text.el().set(text(), self)
    }
}

#[element_component]
pub fn Cursor(hooks: &mut Hooks) -> Element {
    let (show, set_show) = hooks.use_state(true);
    use_interval_deps(hooks, Duration::from_millis(500), false, show, move |show| set_show(!show));
    if show {
        UIBase.el().children(vec![Rectangle.el().set(width(), 2.).set(height(), 13.).set(translation(), vec3(1., 0., 0.))])
    } else {
        Element::new()
    }
}
