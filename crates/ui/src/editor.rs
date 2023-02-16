use std::{fmt::Debug, ops::Deref, sync::Arc};

use closure::closure;
use kiwi_core::{
    name,
    transform::{euler_rotation, scale, translation},
};
use kiwi_ecs::{AttributeConstructor, Component, ComponentAttribute, ComponentEntry, ComponentValue, World};
use kiwi_element::{element_component, Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_renderer::{cast_shadows, color, overlay};
use kiwi_std::{cb, time::Timeout, Cb};
use parking_lot::Mutex;

use crate::{
    align_vertical, space_between_items, Button, ButtonStyle, DialogScreen, DurationEditor, EditableDuration, FlowColumn, FlowRow,
    ScreenContainer, ScrollArea, StylesExt, Text, STREET,
};

#[derive(Clone, Debug)]
pub struct EditorOpts {
    pub enum_can_change_type: bool,
}

impl Default for EditorOpts {
    fn default() -> Self {
        Self { enum_can_change_type: true }
    }
}

pub type ChangeCb<T> = Cb<dyn Fn(T) + Sync + Send>;

pub trait Editor {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element;
    fn edit_or_view(self, on_change: Option<ChangeCb<Self>>, opts: EditorOpts) -> Element
    where
        Self: Sized,
    {
        if let Some(on_change) = on_change {
            self.editor(on_change, opts)
        } else {
            self.view(opts)
        }
    }
    fn view(self, opts: EditorOpts) -> Element
    where
        Self: Sized,
    {
        self.editor(cb(|_| {}), opts)
    }
}

impl Editor for EditableDuration {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        DurationEditor::new(self, on_change).el()
    }
}

impl<T: Editor + 'static> Editor for Box<T> {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        T::editor(*self, cb(move |new_value| (on_change)(Box::new(new_value))), opts)
    }
}

impl<T> Editor for Arc<T>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        T::editor(self.deref().clone(), cb(move |v: T| on_change(Arc::new(v))) as Cb<dyn Fn(T) + Sync + Send>, opts)
    }
}

impl<T> Editor for Arc<Mutex<T>>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        let v: T = self.lock().clone();
        T::editor(
            v,
            cb(move |v: T| {
                // Update the shared value
                *self.lock() = v;
                // Give the same value which is now internally mutated
                on_change(self.clone())
            }),
            opts,
        )
    }
}

impl Editor for () {
    fn editor(self, _on_change: ChangeCb<Self>, _opts: EditorOpts) -> Element {
        Element::new()
    }
}

impl Editor for Timeout {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        DurationEditor::new(
            EditableDuration::new(self.duration(), true, self.duration().as_secs().to_string()),
            cb(move |v| (on_change)(Timeout::new(v.dur()))),
        )
        .el()
    }
}

impl<T: Default + Editor + 'static> Editor for Option<T> {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        if let Some(inner_value) = self {
            FlowRow(vec![
                Button::new("\u{f056}", closure!(clone on_change, |_| on_change.0(None))).style(ButtonStyle::Flat).el(),
                T::editor(inner_value, cb(closure!(clone on_change, |value| on_change.0(Some(value)))), opts),
            ])
            .el()
        } else {
            Button::new("\u{f055}", closure!(clone on_change, |_| on_change.0(Some(T::default())))).style(ButtonStyle::Flat).el()
        }
    }

    fn view(self, opts: EditorOpts) -> Element
    where
        Self: Sized,
    {
        if let Some(value) = self {
            T::view(value, opts)
        } else {
            Text::el("None")
        }
    }
}

type EditFn<T> = fn(ComponentEntryEditor, ChangeCb<T>, EditorOpts) -> Element;
type ViewFn = fn(ComponentEntryEditor, EditorOpts) -> Element;

#[derive(Clone)]
/// Created through the `Editable` attribute
pub struct ComponentEntryEditor {
    pub entry: ComponentEntry,
    edit: EditFn<Self>,
    view: ViewFn,
}

impl ComponentEntryEditor {
    pub fn entry(&self) -> &ComponentEntry {
        &self.entry
    }
}

impl Debug for ComponentEntryEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentEntryEditor").field("entry", &self.entry).finish()
    }
}

impl Editor for ComponentEntryEditor {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        (self.edit)(self, on_change, opts)
    }

    fn view(self, opts: EditorOpts) -> Element {
        (self.view)(self, opts)
    }
}

#[derive(Copy, Clone)]
pub struct Editable {
    edit: EditFn<ComponentEntryEditor>,
    view: ViewFn,
}

impl Editable {
    /// Create an editor for this component entry
    pub fn edit(&self, entry: ComponentEntry) -> ComponentEntryEditor {
        ComponentEntryEditor { entry, edit: self.edit, view: self.view }
    }
}

impl ComponentAttribute for Editable {}

impl<T> AttributeConstructor<T, ()> for Editable
where
    T: ComponentValue + Editor,
{
    fn construct(store: &mut kiwi_ecs::AttributeStore, _: ()) {
        let editable = Editable {
            edit: |editor, on_change, opts| {
                let entry = editor.entry;
                let desc = entry.desc();
                T::editor(
                    entry.into_inner(),
                    cb(move |v| (on_change)(ComponentEntryEditor { entry: ComponentEntry::from_raw_parts(desc, v), ..editor }))
                        as ChangeCb<T>,
                    opts,
                )
            },
            view: |editor, opts| {
                let entry = editor.entry;
                T::view(entry.into_inner(), opts)
            },
        };

        store.set(editable);
    }
}

/// Adds the `Editable` attribute to multiple components where depending on `kiwi_ui` is not
/// possible.
pub fn hydrate_editable() {
    fn set<T: ComponentValue + Editor>(component: Component<T>) {
        let mut store = component.attributes_mut();
        <Editable as AttributeConstructor<T, ()>>::construct(&mut store, ());
    }

    set(translation());
    set(euler_rotation());
    set(scale());
    set(color());
    set(name());

    set(overlay());
    set(cast_shadows());
}

/// Delegates a type editor to edit in a new `screen`
#[derive(Debug, Clone)]
pub struct OffscreenEditor<T> {
    pub value: T,
    pub opts: EditorOpts,
    pub editor: Cb<dyn Fn(T, Option<ChangeCb<T>>, EditorOpts) -> Element + Sync + Send>,
    pub on_confirm: Option<ChangeCb<T>>,
    pub title: String,
}

impl<T: Debug + ComponentValue + Editor> ElementComponent for OffscreenEditor<T> {
    fn render(self: Box<Self>, hooks: &mut kiwi_element::Hooks) -> Element {
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
fn EditorScreen<T: Debug + ComponentValue + Editor>(
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
                .set(align_vertical(), crate::Align::Center),
            ])
            .set(space_between_items(), STREET),
        )
        .el(),
    )
    .el()
}
