use crate::{
    button::{Button, ButtonStyle},
    default_theme::{StylesExt, STREET},
    layout::{FlowColumn, FlowRow},
    screens::{DialogScreen, ScreenContainer},
    scroll_area::ScrollArea,
    text::Text,
};

use super::{ChangeCb, Editor, EditorOpts};
use ambient_cb::{cb, Cb};
use ambient_element::{element_component, Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::layout::{align_vertical_center, space_between_items},
    ecs::ComponentValue,
};
use closure::closure;
use std::fmt::Debug;

/// Delegates a type editor to edit in a new `screen`
#[derive(Debug, Clone)]
pub struct OffscreenEditor<T> {
    pub value: T,
    pub opts: EditorOpts,
    pub editor: Cb<dyn Fn(T, Option<ChangeCb<T>>, EditorOpts) -> Element + Sync + Send>,
    pub on_confirm: Option<ChangeCb<T>>,
    pub title: String,
}

impl<T: Debug + Clone + Sync + Send + 'static + Editor> ElementComponent for OffscreenEditor<T> {
    fn render(self: Box<Self>, hooks: &mut ambient_element::Hooks) -> Element {
        let Self { title, value, on_confirm, editor, opts } = *self;

        let (screen, set_screen) = hooks.use_state(None);

        FlowRow(vec![
            ScreenContainer(screen).el(),
            Button::new("\u{fb4e} Edit", move |_| {
                set_screen(Some(
                    EditorScreen {
                        value: value.clone(),
                        title: title.clone(),
                        edit: on_confirm.is_some(),
                        on_confirm: cb(closure!(clone on_confirm, clone set_screen, |value| {
                            if let Some(on_confirm) = on_confirm.as_ref() {
                                on_confirm(value);
                            }
                            set_screen(None);
                        })),
                        on_cancel: cb(closure!(clone set_screen, || {
                            set_screen(None);
                        })),
                        editor: editor.clone(),
                        opts: opts.clone(),
                    }
                    .el(),
                ));
            })
            .style(ButtonStyle::Flat)
            .el(),
        ])
        .el()
    }
}

#[element_component]
fn EditorScreen<T: Debug + Clone + Sync + Send + 'static + Editor>(
    hooks: &mut Hooks,
    value: T,
    title: String,
    on_confirm: Cb<dyn Fn(T) + Sync + Send>,
    on_cancel: Cb<dyn Fn() + Sync + Send>,
    edit: bool,
    editor: Cb<dyn Fn(T, Option<ChangeCb<T>>, EditorOpts) -> Element + Sync + Send>,
    opts: EditorOpts,
) -> Element {
    let (value, set_value) = hooks.use_state(value);
    DialogScreen(
        ScrollArea(
            FlowColumn::el([
                Text::el(title).header_style(),
                editor(value.clone(), if edit { Some(set_value.clone()) } else { None }, opts),
                FlowRow(vec![
                    Button::new_once("Ok", move |_| on_confirm(value)).style(ButtonStyle::Primary).el(),
                    Button::new_once("Cancel", move |_| on_cancel()).style(ButtonStyle::Flat).el(),
                ])
                .el()
                .set(space_between_items(), STREET)
                .set_default(align_vertical_center()),
            ])
            .set(space_between_items(), STREET),
        )
        .el(),
    )
    .el()
}
