use std::{ops::Deref, sync::Arc, time::Duration};

use elements_core::{
    name, transform::{euler_rotation, scale, translation}
};
use elements_ecs::{AttributeConstructor, Component, ComponentAttribute, ComponentEntry, ComponentValue};
use elements_element::{Element, ElementComponentExt};
use elements_renderer::color;
use elements_std::{time::Timeout, Cb};
use parking_lot::Mutex;
use serde::de::value;

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

/// Represents a type which can construct a UI for editing a contained value.
///
/// When the user changes the value, the callback will be invoked
pub trait Editor {
    type Output;
    /// Construct the UI tree to edit this value
    fn editor(self, on_change: Option<ChangeCb<Self::Output>>, opts: EditorOpts) -> Element;
    /// Returns the current value
    fn value(&self) -> Self::Output;
}

impl Editor for EditableDuration {
    type Output = Duration;
    fn editor(self, on_change: Option<ChangeCb<Self::Output>>, _: EditorOpts) -> Element {
        DurationEditor::new(self, on_change.unwrap_or_else(|| Cb(Arc::new(|_| {})))).el()
    }

    fn value(&self) -> Self::Output {
        self.dur
    }
}

impl<T: Editor + 'static> Editor for Box<T> {
    type Output = T::Output;

    fn editor(self, on_change: Option<ChangeCb<Self::Output>>, opts: EditorOpts) -> Element {
        T::editor(*self, on_change.map(|cb| Cb::new(move |new_value| cb.0(new_value)) as Cb<dyn Fn(T::Output) + Sync + Send>), opts)
    }

    fn value(&self) -> Self::Output {
        self.deref().value()
    }
}

impl<T> Editor for Arc<T>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    type Output = T::Output;

    fn editor(self, on_change: Option<ChangeCb<Self::Output>>, opts: EditorOpts) -> Element {
        T::editor(self.deref().clone(), on_change, opts)
    }

    fn value(&self) -> Self::Output {
        self.deref().value()
    }
}

impl<T> Editor for Arc<Mutex<T>>
where
    T: 'static + Send + Sync + Clone + Editor<Output = T>,
{
    type Output = T;

    fn editor(self, on_change: Option<ChangeCb<Self::Output>>, opts: EditorOpts) -> Element {
        let v: T = self.lock().clone();
        T::editor(
            v,
            on_change.map(|f| {
                Cb::new(move |v: T| {
                    // Update the shared value
                    *self.lock() = v;
                    // Give the same value which is now internally mutated
                    f(v)
                }) as Cb<dyn Fn(T) + Sync + Send>
            }),
            opts,
        )
    }

    fn value(&self) -> Self::Output {
        self.lock().value()
    }
}

impl Editor for () {
    type Output = Self;
    fn editor(self, _on_change: Option<ChangeCb<Self::Output>>, _opts: EditorOpts) -> Element {
        Element::new()
    }

    fn value(&self) -> Self::Output {}
}

impl Editor for Timeout {
    type Output = Self;
    fn editor(self, on_change: Option<ChangeCb<Self::Output>>, _: EditorOpts) -> Element {
        let on_change = on_change.unwrap_or_else(|| Cb::new(|_| {}));

        DurationEditor::new(
            EditableDuration::new(self.duration(), true, self.duration().as_secs().to_string()),
            Cb::new(move |v| (on_change)(Timeout::new(v))),
        )
        .el()
    }

    fn value(&self) -> Self::Output {
        *self
    }
}

impl<T> Editor for Option<T>
where
    T: Editor + 'static,
    T::Output: Default,
{
    type Output = Option<T::Output>;

    fn editor(self, on_change: Option<ChangeCb<Self::Output>>, opts: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            if let Some(inner) = self {
                FlowRow(vec![
                    Button::new("\u{f056}", closure!(clone on_change, |_| on_change.0(None))).style(ButtonStyle::Flat).el(),
                    T::editor(inner, Some(Cb(Arc::new(closure!(clone on_change, |value| on_change.0(Some(value)))))), opts),
                ])
                .el()
            } else {
                Button::new("\u{f055}", closure!(clone on_change, |_| on_change.0(Some(Default::default())))).style(ButtonStyle::Flat).el()
            }
        } else if let Some(value) = self {
            T::editor(value, None, opts)
        } else {
            Text::el("None")
        }
    }

    fn value(&self) -> Self::Output {
        self.map(|v| v.value())
    }
}

type EditFn = fn(ComponentEntry, Option<ChangeCb<ComponentEntry>>, EditorOpts) -> Element;

#[derive(Clone)]
/// Created through the `Editable` attribute
pub struct ComponentEntryEditor {
    entry: ComponentEntry,
    edit: EditFn,
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
    type Output = ComponentEntry;
    fn editor(self, on_change: Option<ChangeCb<Self::Output>>, opts: EditorOpts) -> Element {
        (self.edit)(self.entry, on_change, opts)
    }

    fn value(&self) -> Self::Output {
        self.entry.clone()
    }
}

#[derive(Copy, Clone)]
pub struct Editable {
    edit: EditFn,
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
    T: ComponentValue + Editor<Output = T>,
{
    fn construct(store: &mut elements_ecs::AttributeStore, _: ()) {
        let editable = Editable {
            edit: |entry, on_change, opts| {
                let desc = entry.desc();
                T::editor(
                    entry.into_inner(),
                    on_change.map(|f| Cb::new(move |v| (f)(ComponentEntry::from_raw_parts(desc, v))) as ChangeCb<T::Output>),
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
    fn set<T: ComponentValue + Editor<Output = T>>(component: Component<T>) {
        let mut store = component.attributes_mut();
        <Editable as AttributeConstructor<T, ()>>::construct(&mut store, ());
    }

    set(translation());
    set(euler_rotation());
    set(scale());
    set(color());
    set(name());
}
