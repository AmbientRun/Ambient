use std::{ops::Deref, sync::Arc};

use elements_core::{
    name, transform::{euler_rotation, scale, translation}
};
use elements_ecs::{AttributeConstructor, Component, ComponentAttribute, ComponentEntry, ComponentValue};
use elements_element::{Element, ElementComponentExt};
use elements_renderer::color;
use elements_std::{time::Timeout, Cb};
use parking_lot::Mutex;

use crate::{Button, ButtonStyle, DurationEditor, EditableDuration, FlowRow, Text};

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
        self.editor(Cb::new(|_| {}), opts)
    }
}

impl Editor for EditableDuration {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        DurationEditor::new(self, on_change).el()
    }
}

impl<T: Editor + 'static> Editor for Box<T> {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        T::editor(*self, Cb(Arc::new(move |new_value| (on_change)(Box::new(new_value))) as Arc<dyn Fn(T) + Sync + Send>), opts)
    }
}

impl<T> Editor for Arc<T>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        T::editor(self.deref().clone(), Cb::new(move |v: T| on_change(Arc::new(v))) as Cb<dyn Fn(T) + Sync + Send>, opts)
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
            Cb::new(move |v: T| {
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
            Cb::new(move |v| (on_change)(Timeout::new(v.dur()))),
        )
        .el()
    }
}

impl<T: Default + Editor + 'static> Editor for Option<T> {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        if let Some(inner_value) = self {
            FlowRow(vec![
                Button::new("\u{f056}", closure!(clone on_change, |_| on_change.0(None))).style(ButtonStyle::Flat).el(),
                T::editor(inner_value, Cb(Arc::new(closure!(clone on_change, |value| on_change.0(Some(value))))), opts),
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

#[derive(Clone)]
/// Created through the `Editable` attribute
pub struct ComponentEntryEditor {
    pub entry: ComponentEntry,
    edit: EditFn<Self>,
}

impl ComponentEntryEditor {
    pub fn entry(&self) -> &ComponentEntry {
        &self.entry
    }
}

impl std::fmt::Debug for ComponentEntryEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentEntryEditor").field("entry", &self.entry).finish()
    }
}

impl Editor for ComponentEntryEditor {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        (self.edit)(self, on_change, opts)
    }
}

#[derive(Copy, Clone)]
pub struct Editable {
    edit: EditFn<ComponentEntryEditor>,
}

impl Editable {
    /// Create an editor for this component entry
    pub fn edit(&self, entry: ComponentEntry) -> ComponentEntryEditor {
        ComponentEntryEditor { entry, edit: self.edit }
    }
}

impl ComponentAttribute for Editable {}

impl<T> AttributeConstructor<T, ()> for Editable
where
    T: ComponentValue + Editor,
{
    fn construct(store: &mut elements_ecs::AttributeStore, _: ()) {
        let editable = Editable {
            edit: |editor, on_change, opts| {
                let entry = editor.entry;
                let desc = entry.desc();
                T::editor(
                    entry.into_inner(),
                    Cb::new(move |v| (on_change)(ComponentEntryEditor { entry: ComponentEntry::from_raw_parts(desc, v), ..editor }))
                        as ChangeCb<T>,
                    opts,
                )
            },
        };

        store.set(editable);
    }
}

/// Adds the `Editable` attribute to multiple components where depending on `elements_ui` is not
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
}
