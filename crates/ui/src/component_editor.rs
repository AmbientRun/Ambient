use ambient_core::{
    name,
    transform::{euler_rotation, scale, translation},
};
use ambient_ecs::{AttributeConstructor, Component, ComponentAttribute, ComponentEntry, ComponentValue};
use ambient_element::Element;
use ambient_renderer::{cast_shadows, color, overlay};
use ambient_std::cb;
use ambient_ui_components::editor::{ChangeCb, Editor, EditorOpts};

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

impl std::fmt::Debug for ComponentEntryEditor {
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
    fn construct(store: &mut ambient_ecs::AttributeStore, _: ()) {
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

/// Adds the `Editable` attribute to multiple components where depending on `ambient_ui` is not
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
