use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Deref, sync::Arc};

use closure::closure;
use elements_core::on_window_event;
use elements_ecs::{EntityId, World};
use elements_element::{element_component, Element, ElementComponent, ElementComponentExt, Hooks};
use elements_input::{on_app_keyboard_input, KeyboardEvent};
use elements_std::{color::Color, Cb};
use indexmap::IndexMap;
use itertools::Itertools;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};

use super::{Button, ButtonStyle, Dropdown, Editor, EditorOpts, FlowColumn, FlowRow, Focus, UIBase, UIExt};
use crate::{layout::*, StylesExt, COLLECTION_ADD_ICON, COLLECTION_DELETE_ICON, MOVE_DOWN_ICON, MOVE_UP_ICON, STREET};

#[element_component]
pub fn ListEditor<T: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static>(
    _world: &mut World,
    _: &mut Hooks,
    value: Vec<T>,
    on_change: Option<Cb<dyn Fn(Vec<T>) + Sync + Send>>,
) -> Element {
    if let Some(on_change) = on_change {
        let button_size = 20.;
        FlowColumn::el([
            FlowColumn(
                value
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        FlowRow(vec![
                            Button::new(
                                COLLECTION_DELETE_ICON,
                                closure!(clone on_change, clone value, |_| {
                                    let mut value = value.clone();
                                    value.remove(i);
                                    on_change.0(value);
                                }),
                            )
                            .style(ButtonStyle::Flat)
                            .el()
                            .set(min_width(), button_size),
                            if i > 0 {
                                Button::new(
                                    MOVE_UP_ICON,
                                    closure!(clone on_change, clone value, |_| {
                                        let mut value = value.clone();
                                        value.swap(i, i - 1);
                                        on_change.0(value);
                                    }),
                                )
                                .style(ButtonStyle::Flat)
                                .el()
                                .set(min_width(), button_size)
                            } else {
                                UIBase.el().set(width(), button_size).set(height(), 1.)
                            },
                            if i < value.len() - 1 {
                                Button::new(
                                    MOVE_DOWN_ICON,
                                    closure!(clone on_change, clone value, |_| {
                                        let mut value = value.clone();
                                        value.swap(i, i + 1);
                                        on_change.0(value);
                                    }),
                                )
                                .style(ButtonStyle::Flat)
                                .el()
                                .set(min_width(), button_size)
                            } else {
                                UIBase.el().set(width(), button_size).set(height(), 1.)
                            },
                            T::editor(
                                item.clone(),
                                Some(Cb(Arc::new(closure!(clone value, clone on_change, |item| {
                                    let mut value = value.clone();
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
                    let mut value = value.clone();
                    value.push(T::default());
                    on_change.0(value);
                }),
            )
            .style(ButtonStyle::Flat)
            .el(),
        ])
    } else {
        unimplemented!()
    }
}

impl<T: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static> Editor for Vec<T> {
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        ListEditor { value: self, on_change }.el()
    }
}

#[derive(Debug, Clone)]
pub struct MinimalListEditor<T: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static> {
    pub value: Vec<T>,
    pub on_change: Option<Cb<dyn Fn(Vec<T>) + Sync + Send>>,
    pub item_opts: EditorOpts,
    pub add_presets: Option<Vec<T>>,
    pub add_title: String,
}
impl<T: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static> ElementComponent for MinimalListEditor<T> {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        MinimalListEditorWithItemEditor {
            value: self.value,
            on_change: self.on_change,
            item_opts: self.item_opts,
            add_presets: self.add_presets,
            add_title: self.add_title,
            item_editor: Cb(Arc::new(T::editor)),
        }
        .el()
    }
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub struct MinimalListEditorWithItemEditor<T: std::fmt::Debug + Clone + Default + Sync + Send + 'static> {
    pub value: Vec<T>,
    pub on_change: Option<Cb<dyn Fn(Vec<T>) + Sync + Send>>,
    pub item_opts: EditorOpts,
    pub add_presets: Option<Vec<T>>,
    pub add_title: String,
    pub item_editor: Cb<dyn Fn(T, Option<Cb<dyn Fn(T) + Sync + Send>>, EditorOpts) -> Element + Sync + Send>,
}
impl<T: std::fmt::Debug + Clone + Default + Sync + Send + 'static> ElementComponent for MinimalListEditorWithItemEditor<T> {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let Self { value, on_change, item_opts, add_presets, add_title, item_editor } = *self;
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
                                    let mut value = value.clone();
                                    value[i] = item;
                                    on_change.0(value);
                                })) as Arc<dyn Fn(T) + Sync + Send>)
                            }),
                            on_delete: on_change.clone().map(|on_change| {
                                Cb(Arc::new(closure!(clone value, clone on_change, || {
                                    let mut value = value.clone();
                                    value.remove(i);
                                    on_change.0(value);
                                })) as Arc<dyn Fn() + Sync + Send>)
                            }),
                            item_opts: item_opts.clone(),
                            item_editor: item_editor.clone(),
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
                                    item_editor.0(item.clone(), None, Default::default())
                                        .on_mouse_down(closure!(clone value, clone on_change, |_, _, _| {
                                            let mut value = value.clone();
                                            value.push(item.clone());
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
                            let mut value = value.clone();
                            value.push(T::default());
                            on_change.0(value);
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
pub struct MinimalListEditorItem<T: std::fmt::Debug + Clone + Default + Sync + Send + 'static> {
    pub value: T,
    pub on_change: Option<Cb<dyn Fn(T) + Sync + Send>>,
    pub on_delete: Option<Cb<dyn Fn() + Sync + Send>>,
    pub item_opts: EditorOpts,
    pub item_editor: Cb<dyn Fn(T, Option<Cb<dyn Fn(T) + Sync + Send>>, EditorOpts) -> Element + Sync + Send>,
}
impl<T: std::fmt::Debug + Clone + Default + Sync + Send + 'static> ElementComponent for MinimalListEditorItem<T> {
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
    K: Editor + std::fmt::Debug + Clone + Default + Hash + PartialEq + Eq + PartialOrd + Ord + Sync + Send + 'static,
    V: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static,
> {
    pub value: HashMap<K, V>,
    pub on_change: Option<Cb<dyn Fn(HashMap<K, V>) + Sync + Send>>,
}
impl<
        K: Editor + std::fmt::Debug + Clone + Default + Hash + PartialEq + Eq + PartialOrd + Ord + Sync + Send + 'static,
        V: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static,
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
                                        let mut value = value.clone();
                                        let item = value.remove(&key).unwrap();
                                        value.insert(new_key, item);
                                        on_change.0(value);
                                    })) as Arc<dyn Fn(K) + Sync + Send>)
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
                                    })) as Arc<dyn Fn(V) + Sync + Send>)
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
        K: Editor + std::fmt::Debug + Clone + Default + Hash + PartialEq + Eq + PartialOrd + Ord + Sync + Send + 'static,
        V: Editor + std::fmt::Debug + Clone + Default + Sync + Send + 'static,
    > Editor for HashMap<K, V>
{
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        KeyValueEditor { value: self, on_change }.el()
    }
}

#[derive(Debug, Clone)]
pub struct IndexMapEditor<K, V> {
    value: Arc<IndexMap<K, V>>,
    on_change: Cb<dyn Fn(IndexMap<K, V>) + Send + Sync>,
    use_row_instead_of_column: bool,
}

impl<K, V> IndexMapEditor<K, V> {
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
    K: std::hash::Hash + Eq + Send + Sync + Debug + 'static + Clone + Editor + Default,
    V: Send + Sync + Debug + 'static + Clone + Editor + Default,
{
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Send + Sync>>, opts: EditorOpts) -> Element {
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
}

#[derive(Debug, Clone)]
struct IndexMapEntryPart<K, V> {
    key: K,
    value: V,
    parent: Arc<IndexMap<K, V>>,
    on_change: Cb<dyn Fn(IndexMap<K, V>) + Send + Sync>,
}

impl<K, V> ElementComponent for IndexMapEntryPart<K, V>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static + Editor,
    V: Clone + Debug + Editor + Send + Sync + 'static,
{
    fn render(self: Box<Self>, _world: &mut World, _: &mut Hooks) -> Element {
        let Self { key, value, on_change, parent } = *self;

        let key_editor = {
            let parent = parent.clone();
            let on_change = on_change.clone();
            let old_key = key.clone();
            K::editor(
                key.clone(),
                Some(Cb(Arc::new(move |key| {
                    let mut map = parent.deref().clone();
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
            V::editor(
                value,
                Some(Cb(Arc::new(move |value| {
                    let mut map = parent.deref().clone();
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
