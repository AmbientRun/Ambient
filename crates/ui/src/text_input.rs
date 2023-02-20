use std::{self, sync::Arc, time::Duration};

use ambient_core::{transform::translation, window};
use ambient_ecs::EntityId;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_input::{on_app_keyboard_input, on_app_received_character, KeyboardEvent};
use ambient_renderer::color;
use ambient_std::{cb, Cb};
use closure::closure;
use glam::*;
use winit::{
    event::{ElementState, VirtualKeyCode},
    window::CursorIcon,
};

use super::{Editor, EditorOpts, Focus, Text, UIExt};
use crate::{layout::*, text, use_interval_deps, Rectangle, UIBase};

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
    let el = if value.is_empty() && !focused && placeholder.is_some() {
        Text.el().set(text(), placeholder.unwrap()).set(color(), vec4(1., 1., 1., 0.2))
    } else {
        Text.el()
            .set(text(), if password { value.chars().map(|_| '*').collect() } else { value.clone() })
            .set(color(), vec4(0.9, 0.9, 0.9, 1.))
    }
    .init(layout(), Layout::Flow)
    .set(fit_horizontal(), Fit::None)
    .set(fit_vertical(), Fit::None)
    .set(min_width(), 3.)
    .set(min_height(), 13.)
    .on_spawned(move |_, id| set_self_id(id))
    .on_mouse_up(move |_, id, _| {
        set_focus(Focus(Some(id)));
    })
    .on_mouse_enter(|world, _| world.resource(window()).set_cursor_icon(CursorIcon::Text))
    .on_mouse_leave(|world, _| world.resource(window()).set_cursor_icon(CursorIcon::Default));

    if focused {
        el.set(align_horizontal(), Align::End)
            .children(vec![Cursor.el()])
            .listener(
                on_app_received_character(),
                Arc::new(closure!(clone value, clone on_change, clone on_submit, |_, _, c| {
                    if command {
                        return true;
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
                    true
                })),
            )
            .listener(
                on_app_keyboard_input(),
                Arc::new(move |_, _, event| {
                    if let KeyboardEvent { keycode: Some(kc), state, .. } = event {
                        match kc {
                            VirtualKeyCode::LWin => {
                                #[cfg(target_os = "macos")]
                                set_command(state == &ElementState::Pressed);
                            }
                            VirtualKeyCode::LControl => {
                                #[cfg(not(target_os = "macos"))]
                                set_command(state == &ElementState::Pressed);
                            }
                            VirtualKeyCode::V => {
                                if command && state == &ElementState::Pressed {
                                    if let Ok(paste) = arboard::Clipboard::new().unwrap().get_text() {
                                        on_change.0(format!("{value}{paste}"));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    true
                }),
            )
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
