use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Deref, sync::Arc};

use closure::closure;
use elements_core::on_window_event;
use elements_ecs::{ComponentValue, EntityId, World};
use elements_element::{element_component, Element, ElementComponent, ElementComponentExt, Hooks};
use elements_input::{on_app_keyboard_input, KeyboardEvent};
use elements_std::{color::Color, Cb};
use indexmap::IndexMap;
use itertools::Itertools;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};

use super::{Button, ButtonStyle, Dropdown, Editor, EditorOpts, FlowColumn, FlowRow, Focus, UIBase, UIExt};
use crate::{layout::*, StylesExt, COLLECTION_ADD_ICON, COLLECTION_DELETE_ICON, MOVE_DOWN_ICON, MOVE_UP_ICON, STREET};

#[element_component]
pub fn ListEditor<T>(
    _world: &mut World,
    _: &mut Hooks,
    editor: Vec<T>,
    on_change: Option<Cb<dyn Fn(Vec<T::Output>) + Sync + Send>>,
) -> Element
where
    T: Editor + Debug + Clone + Default + ComponentValue,
    T::Output: Debug + Clone + ComponentValue,
{
    if let Some(on_change) = on_change {
        let button_size = 20.;
        FlowColumn::el([
            FlowColumn(
                editor
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        FlowRow(vec![
                            Button::new(
                                COLLECTION_DELETE_ICON,
                                closure!(clone on_change, clone editor, |_| {
                                    editor.remove(i);
                                    on_change.0(editor.value());
                                }),
                            )
                            .style(ButtonStyle::Flat)
                            .el()
                            .set(min_width(), button_size),
                            if i > 0 {
                                Button::new(
                                    MOVE_UP_ICON,
                                    closure!(clone on_change, clone editor, |_| {
                                        editor.swap(i, i - 1);
                                        on_change.0(editor.value());
                                    }),
                                )
                                .style(ButtonStyle::Flat)
                                .el()
                                .set(min_width(), button_size)
                            } else {
                                UIBase.el().set(width(), button_size).set(height(), 1.)
                            },
                            if i < editor.len() - 1 {
                                Button::new(
                                    MOVE_DOWN_ICON,
                                    closure!(clone on_change, clone editor, |_| {
                                        editor.swap(i, i + 1);
                                        on_change.0(editor.value());
                                    }),
                                )
                                .style(ButtonStyle::Flat)
                                .el()
                                .set(min_width(), button_size)
                            } else {
                                UIBase.el().set(width(), button_size).set(height(), 1.)
                            },
                            item.editor(
                                Some(Cb(Arc::new(closure!(clone editor, clone on_change, |item| {
                                    let mut value = editor.value();
                                    value[i] = item;
                                    on_change.0(value);
                                })))),
                                Default::default(),
                            ),
                        ])
                        .el()
                    })
                    .collect(),
            )
            .el(),
            Button::new(
                COLLECTION_ADD_ICON,
                closure!(clone on_change, |_| {
                    editor.push(Default::default());
                    on_change.0(editor.value());
                }),
            )
            .style(ButtonStyle::Flat)
            .el(),
        ])
    } else {
        unimplemented!()
    }
}

impl<T: Editor + Debug + Clone + Default + Sync + Send + 'static> Editor for Vec<T>
where
    T: Editor + Debug + Clone + Default + ComponentValue,
    T::Output: Debug + Clone + ComponentValue,
{
    type Output = Vec<T::Output>;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self::Output) + Sync + Send>>, _: EditorOpts) -> Element {
        ListEditor { editor: self, on_change }.el()
    }

    fn value(&self) -> Self::Output {
        self.iter().map(|v| v.value()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct MinimalListEditor<T: Editor + Debug + Clone + Default + Sync + Send + 'static> {
    pub editor: Vec<T>,
    pub on_change: Option<Cb<dyn Fn(Vec<T>) + Sync + Send>>,
    pub item_opts: EditorOpts,
    pub add_presets: Option<Vec<T>>,
    pub add_title: String,
}
impl<T: Editor + Debug + Clone + Default + Sync + Send + 'static> ElementComponent for MinimalListEditor<T> {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        MinimalListEditorWithItemEditor {
            value: self.value,
            on_change: self.on_change,
            item_opts: self.item_opts,
            add_presets: self.add_presets,
            add_title: self.add_title,
        }
        .el()
    }
}

#[allow(clippy::type_complexity)]
pub struct MinimalListEditorWithItemEditor<T: Editor + Debug + Clone + Default + Sync + Send + 'static> {
    pub value: Vec<T>,
    pub on_change: Option<Cb<dyn Fn(Vec<T::Output>) + Sync + Send>>,
    pub item_opts: EditorOpts,
    pub add_presets: Option<Vec<T>>,
    pub add_title: String,
}

impl<T> Clone for MinimalListEditorWithItemEditor<T>
where
    T: Editor + Debug + Clone + Default + Sync + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            on_change: self.on_change.clone(),
            item_opts: self.item_opts.clone(),
            add_presets: self.add_presets.clone(),
            add_title: self.add_title.clone(),
        }
    }
}

impl<T: Editor + Debug + Clone + Default + Sync + Send + 'static> Debug for MinimalListEditorWithItemEditor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinimalListEditorWithItemEditor")
            .field("value", &self.value)
            .field("on_change", &self.on_change)
            .field("item_opts", &self.item_opts)
            .field("add_presets", &self.add_presets)
            .field("add_title", &self.add_title)
            .finish()
    }
}
impl<T> ElementComponent for MinimalListEditorWithItemEditor<T>
where
    T: Editor + Debug + Default + ComponentValue,
    T::Output: Debug + Clone + ComponentValue,
{
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let Self { value, on_change, item_opts, add_presets, add_title } = *self;
        let (add_action, set_add_action) = hooks.use_state(false);
        FlowColumn::el([
            FlowColumn(
                value
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        MinimalListEditorItem {
                            value: item.clone(),
                            on_change: on_change.clone().map(|on_change| {
                                Cb(Arc::new(closure!(clone value, clone on_change, |item| {
                                    let mut value = value.value();
                                    value[i] = item;
                                    on_change.0(value);
                                })) as Arc<dyn Fn(T::Output) + Sync + Send>)
                            }),
                            on_delete: on_change.clone().map(|on_change| {
                                Cb(Arc::new(closure!(clone value, clone on_change, || {
                                    let mut value = value.value();
                                    value.remove(i);
                                    on_change.0(value);
                                })) as Arc<dyn Fn() + Sync + Send>)
                            }),
                            item_opts: item_opts.clone(),
                        }
                        .el()
                    })
                    .collect(),
            )
            .el()
            .set(fit_horizontal(), Fit::Parent),
            if let Some(on_change) = on_change {
                if let Some(add_presets) = add_presets {
                    Dropdown {
                        content: Button::new(
                            add_title,
                            closure!(clone set_add_action, |_| {
                                set_add_action(true);
                            }),
                        )
                        .style(ButtonStyle::Flat)
                        .el(),
                        dropdown: FlowColumn(
                            add_presets
                                .into_iter()
                                .map(move |item| {
                                    item.editor(None, Default::default())
                                        .on_mouse_down(closure!(clone value, clone on_change, |_, _, _| {
                                            let mut value = value.value();
                                            value.push(item.value());
                                            on_change.0(value);
                                        }))
                                        .set(margin(), Borders::even(STREET))
                                })
                                .collect(),
                        )
                        .el()
                        .with_background(Color::rgba(0.05, 0.05, 0.05, 1.))
                        .set(fit_horizontal(), Fit::None)
                        .set(width(), 400.),
                        show: add_action,
                    }
                    .el()
                    .set(margin(), Borders::top(STREET))
                    .listener(
                        on_window_event(),
                        Arc::new(move |_, _, event| {
                            if let WindowEvent::MouseInput { state: ElementState::Pressed, .. } = event {
                                set_add_action(false);
                            }
                        }),
                    )
                } else {
                    Button::new(
                        add_title,
                        closure!(clone value, clone on_change, |_| {
                            let mut editor = value.clone();
                            editor.push(T::default());
                            on_change.0(editor.value());
                        }),
                    )
                    .style(ButtonStyle::Flat)
                    .el()
                }
            } else {
                Element::new()
            },
        ])
    }
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub struct MinimalListEditorItem<T: Editor + Debug + Clone + Default + Sync + Send + 'static> {
    pub value: T,
    pub on_change: Option<Cb<dyn Fn(T::Output) + Sync + Send>>,
    pub on_delete: Option<Cb<dyn Fn() + Sync + Send>>,
    pub item_opts: EditorOpts,
}
impl<T> ElementComponent for MinimalListEditorItem<T>
where
    T: Editor + Debug + Default + ComponentValue,
    T::Output: Debug + Clone + ComponentValue,
{
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let Self { value, on_change, on_delete, item_opts, item_editor } = *self;
        let (self_id, set_self_id) = hooks.use_state(EntityId::null());
        let (focus, set_focus) = hooks.consume_context::<Focus>().expect("No FocusRoot found");
        let focused = focus == Focus(Some(self_id));
        let item = FlowRow(vec![
            UIBase
                .el()
                .set(width(), 5.)
                .set(fit_vertical(), Fit::Parent)
                .with_background(if focused { Color::rgba(0.0, 1., 0., 1.) } else { Color::rgba(0.5, 0.5, 0.5, 1.) })
                .set(margin(), Borders::right(5.)),
            item_editor.0(value, on_change, item_opts).set(fit_horizontal(), Fit::Parent),
        ])
        .el()
        .on_spawned(move |_, id| set_self_id(id))
        .on_mouse_down(move |_, id, _| {
            set_focus(Focus(Some(id)));
        })
        .set(padding(), Borders::vertical(STREET))
        .set(fit_horizontal(), Fit::Parent);
        if focused {
            if let Some(on_delete) = on_delete {
                item.listener(
                    on_app_keyboard_input(),
                    Arc::new(move |_, _, event| {
                        if let KeyboardEvent { keycode: Some(keycode), state: ElementState::Pressed, .. } = event {
                            if *keycode == VirtualKeyCode::Back || *keycode == VirtualKeyCode::Delete {
                                on_delete.0();
                                return true;
                            }
                        }
                        false
                    }),
                )
            } else {
                item
            }
        } else {
            item
        }
    }
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub struct KeyValueEditor<
    K: Editor + Debug + Clone + Default + Hash + PartialEq + Eq + PartialOrd + Ord + Sync + Send + 'static,
    V: Editor + Debug + Clone + Default + Sync + Send + 'static,
> {
    pub value: HashMap<K, V>,
    pub on_change: Option<Cb<dyn Fn(HashMap<K, V>) + Sync + Send>>,
}
impl<
        K: Editor + Debug + Clone + Default + Hash + PartialEq + Eq + PartialOrd + Ord + Sync + Send + 'static,
        V: Editor + Debug + Clone + Default + Sync + Send + 'static,
    > ElementComponent for KeyValueEditor<K, V>
{
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        FlowColumn::el([
            FlowColumn(
                value
                    .clone()
                    .into_iter()
                    .sorted_by_key(|(key, _)| key.clone())
                    .map(closure!(clone value, clone on_change, |(key, item)| {
                        FlowRow(vec![
                            K::editor(
                                key.clone(),
                                on_change.clone().map(|on_change| {
                                    Cb(Arc::new(closure!(clone key, clone on_change, clone value, |new_key| {
                                        let mut value = value.value();
                                        let item = value.remove(&key).unwrap();
                                        value.insert(new_key, item);
                                        on_change.0(value);
                                    })) as Arc<dyn Fn(K::Output) + Sync + Send>)
                                }),
                                Default::default(),
                            ),
                            V::editor(
                                item,
                                on_change.clone().map(|on_change| {
                                    Cb(Arc::new(closure!(clone value, clone on_change, |item| {
                                        let mut value = value.clone();
                                        value.insert(key.clone(), item);
                                        on_change.0(value);
                                    })) as Arc<dyn Fn(V::Output) + Sync + Send>)
                                }),
                                Default::default(),
                            ),
                        ])
                        .el()
                    }))
                    .collect(),
            )
            .el(),
            if let Some(on_change) = on_change {
                Button::new("Add", move |_| {
                    let mut value = value.clone();
                    value.insert(Default::default(), Default::default());
                    on_change(value);
                })
                .el()
            } else {
                Element::new()
            },
        ])
    }
}
impl<
        K: Editor + Debug + Clone + Default + Hash + PartialEq + Eq + PartialOrd + Ord + Sync + Send + 'static,
        V: Editor + Debug + Clone + Default + Sync + Send + 'static,
    > Editor for HashMap<K, V>
{
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        KeyValueEditor { value: self, on_change }.el()
    }

    fn value(&self) -> Self::Output {
        self.clone()
    }
}

pub struct IndexMapEditor<K: Editor, V: Editor> {
    value: Arc<IndexMap<K, V>>,
    on_change: Cb<dyn Fn(IndexMap<K::Output, V::Output>) + Send + Sync>,
    use_row_instead_of_column: bool,
}

impl<K: Editor + Clone, V: Editor + Clone> Clone for IndexMapEditor<K, V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            on_change: self.on_change.clone(),
            use_row_instead_of_column: self.use_row_instead_of_column.clone(),
        }
    }
}

impl<K: Editor + Debug, V: Editor + Debug> Debug for IndexMapEditor<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexMapEditor")
            .field("value", &self.value)
            .field("on_change", &self.on_change)
            .field("use_row_instead_of_column", &self.use_row_instead_of_column)
            .finish()
    }
}

impl<K: Editor, V: Editor> IndexMapEditor<K, V> {
    pub fn new(value: IndexMap<K, V>, on_change: Cb<dyn Fn(IndexMap<K, V>) + Send + Sync>, use_row_instead_of_column: bool) -> Self {
        Self { value: Arc::new(value), on_change, use_row_instead_of_column }
    }
}
impl<K, V> ElementComponent for IndexMapEditor<K, V>
where
    K: Hash + Eq + Send + Sync + Debug + 'static + Clone + Editor + Default,
    V: Send + Sync + Debug + 'static + Clone + Editor + Default,
{
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        let fields = self.value.iter().map(|(key, value)| {
            IndexMapEntryPart { key: key.clone(), value: value.clone(), parent: self.value.clone(), on_change: self.on_change.clone() }.el()
        });

        let map = self.value.clone();
        let on_change = self.on_change.clone();

        let add = Button::new(COLLECTION_ADD_ICON, move |_w| {
            let mut map = map.deref().clone();
            let key = Default::default();
            map.remove(&key);
            map.insert(key, Default::default());
            on_change(map)
        })
        .style(ButtonStyle::Flat)
        .el();

        let fields = fields.chain([add]).collect_vec();
        if self.use_row_instead_of_column { FlowRow(fields).el() } else { FlowColumn(fields).el() }.set(space_between_items(), STREET)
    }
}

/// Editor is implemented for IndexMap and not HashMap since order needs to be
/// preserved
impl<K, V> Editor for IndexMap<K, V>
where
    K: Hash + Eq + Send + Sync + Debug + 'static + Clone + Editor + Default,
    V: Send + Sync + Debug + 'static + Clone + Editor + Default,
    K::Output: Hash + Eq,
{
    type Output = IndexMap<K::Output, V::Output>;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self::Output) + Send + Sync>>, opts: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            IndexMapEditor::new(self, on_change, false).el()
        } else {
            let fields = self
                .into_iter()
                .map(|(k, v)| FlowColumn(vec![K::editor(k, None, opts.clone()), V::editor(v, None, opts.clone())]).el())
                .collect_vec();

            FlowColumn(fields).el().set(space_between_items(), STREET)
        }
    }

    fn value(&self) -> Self::Output {
        self.iter().map(|(k, v)| (k.value(), v.value())).collect()
    }
}

struct IndexMapEntryPart<K: Editor, V: Editor> {
    key: K,
    value: V,
    parent: Arc<IndexMap<K, V>>,
    on_change: Cb<dyn Fn(IndexMap<K::Output, V::Output>) + Send + Sync>,
}

impl<K: Editor + Clone, V: Editor + Clone> Clone for IndexMapEntryPart<K, V> {
    fn clone(&self) -> Self {
        Self { key: self.key.clone(), value: self.value.clone(), parent: self.parent.clone(), on_change: self.on_change.clone() }
    }
}

impl<K: Editor, V: Editor> IndexMapEntryPart<K, V> {
    fn new(key: K, value: V, parent: Arc<IndexMap<K, V>>, on_change: Cb<dyn Fn(IndexMap<K::Output, V::Output>) + Send + Sync>) -> Self {
        Self { key, value, parent, on_change }
    }
}

impl<K: Editor + Debug, V: Editor + Debug> Debug for IndexMapEntryPart<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexMapEntryPart")
            .field("key", &self.key)
            .field("value", &self.value)
            .field("parent", &self.parent)
            .field("on_change", &self.on_change)
            .finish()
    }
}

impl<K, V> ElementComponent for IndexMapEntryPart<K, V>
where
    K: 'static + Default + Hash + Eq + Clone + Debug + Send + Sync + Editor,
    V: 'static + Default + Clone + Debug + Editor + Send + Sync,
{
    fn render(self: Box<Self>, _world: &mut World, _: &mut Hooks) -> Element {
        let Self { key, value, on_change, parent } = *self;

        let key_editor = {
            let parent = parent.clone();
            let on_change = on_change.clone();
            let old_key = key.clone();
            key.editor(
                Some(Cb(Arc::new(move |key| {
                    let mut map = parent.deref().value();
                    let val = map.remove(&old_key).expect("Missing key in map");
                    map.insert(key, val);
                    on_change(map);
                }))),
                Default::default(),
            )
        };

        let value_editor = {
            let key = key.clone();
            let on_change = on_change.clone();
            let parent = parent.clone();
            value.editor(
                Some(Cb(Arc::new(move |value| {
                    let mut map = parent.value();
                    map.insert(key.clone(), value);

                    on_change(map)
                }))),
                Default::default(),
            )
        };

        let discard = {
            let map = parent;
            Button::new(COLLECTION_DELETE_ICON, move |_| {
                let mut map = map.deref().clone();
                map.remove(&key).expect("Can not remove non existent element");
                on_change(map)
            })
            .style(ButtonStyle::Flat)
            .el()
        };

        FlowColumn(vec![FlowRow(vec![discard, key_editor]).el().set(space_between_items(), STREET), value_editor])
            .el()
            .panel()
            .set(space_between_items(), STREET)
            .set(padding(), Borders::even(STREET))
    }
}
