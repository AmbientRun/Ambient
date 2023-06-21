use std::{sync::Arc, time::Duration};

use ambient_core::name;
use ambient_ecs::{query, EntityId, World};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_layout::{fit_horizontal, max_width, width};
use ambient_renderer::color;
use ambient_std::{cb, Cb};
use ambient_ui_native::{
    margin, Borders, Button, ButtonStyle, FlowColumn, FlowRow, Text, TextEditor, UIExt,
    CHEVRON_DOWN, CHEVRON_RIGHT, STREET,
};
use glam::vec4;
use itertools::Itertools;
pub trait InspectableWorld: Sync + Send + std::fmt::Debug {
    fn get_entities(
        &self,
        parent: Option<EntityId>,
        cb: Cb<dyn Fn(Vec<InspectedEntity>) + Sync + Send>,
        filter: String,
    );
    fn get_components(
        &self,
        entity: EntityId,
        cb: Cb<dyn Fn(Vec<InspectedComponent>) + Sync + Send>,
        filter: String,
    );
}
#[derive(Debug, Clone)]
pub struct InspectedEntity {
    pub id: EntityId,
    pub name: Option<String>,
}
#[derive(Debug, Clone)]
pub struct InspectedComponent {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct InspectableAsyncWorld(pub Cb<dyn Fn(Cb<dyn Fn(&World) + Sync + Send>) + Sync + Send>);
impl InspectableWorld for InspectableAsyncWorld {
    fn get_entities(
        &self,
        parent: Option<ambient_ecs::EntityId>,
        callback: ambient_std::Cb<dyn Fn(Vec<InspectedEntity>) + Sync + Send>,
        filter: String,
    ) {
        (self.0)(cb(move |world| {
            let entities = if let Some(parent) = parent {
                query(ambient_core::hierarchy::parent())
                    .collect_cloned(world, None)
                    .into_iter()
                    .filter_map(|(id, this_parent)| {
                        if this_parent == parent {
                            Some(InspectedEntity {
                                id,
                                name: world.get_ref(id, name()).map(|x| x.clone()).ok(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect_vec()
            } else {
                query(())
                    .excl(ambient_core::hierarchy::parent())
                    .collect_cloned(world, None)
                    .into_iter()
                    .map(|(id, _)| InspectedEntity {
                        id,
                        name: world.get_ref(id, name()).map(|x| x.clone()).ok(),
                    })
                    .collect_vec()
            };
            let entities = entities
                .into_iter()
                .filter(|inspect| {
                    if filter.is_empty() {
                        return true;
                    }
                    if let Some(name) = &inspect.name {
                        if name.contains(&filter) {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                })
                .collect_vec();
            callback(entities);
        }));
    }

    fn get_components(
        &self,
        entity: EntityId,
        callback: Cb<dyn Fn(Vec<InspectedComponent>) + Sync + Send>,
        filter: String,
    ) {
        (self.0)(cb(move |world: &World| {
            let comps = if let Ok(comps) = world.get_components(entity) {
                comps
                    .into_iter()
                    .map(|comp| InspectedComponent {
                        name: comp.path(),
                        value: format!("{:?}", world.get_entry(entity, comp).unwrap().as_debug()),
                    })
                    .filter(|inspect| {
                        if filter.is_empty() {
                            return true;
                        }
                        if inspect.name.contains(&filter) {
                            true
                        } else {
                            false
                        }
                    })
                    .collect_vec()
            } else {
                Vec::new()
            };
            callback(comps);
        }));
    }
}

#[element_component]
pub fn ECSEditor(hooks: &mut Hooks, world: Arc<dyn InspectableWorld>) -> Element {
    let (comp_filter, set_comp_filter) = hooks.use_state("".to_string());
    let (entity_filter, set_entity_filter) = hooks.use_state("".to_string());

    FlowColumn::el([
        {
            let entity_filter = entity_filter.clone();
            TextEditor::new(entity_filter, set_entity_filter)
                .placeholder(Some("\u{f422} entity filter".to_string()))
                .el()
                .with(margin(), Borders::even(STREET).into())
        },
        {
            let comp_filter = comp_filter.clone();
            TextEditor::new(comp_filter, set_comp_filter)
                .placeholder(Some("\u{f422} component filter".to_string()))
                .el()
                .with(margin(), Borders::even(STREET).into())
        },
        EntityList {
            world,
            parent: None,
            filter_components: comp_filter,
            filter_entities: entity_filter,
        }
        .el(),
    ])
}

#[element_component]
fn EntityList(
    hooks: &mut Hooks,
    world: Arc<dyn InspectableWorld>,
    parent: Option<EntityId>,
    filter_entities: String,
    filter_components: String,
) -> Element {
    let (show_all, set_show_all) = hooks.use_state(false);
    let (entities, set_entities) = hooks.use_state(Vec::new());
    const MAX: usize = 30;
    hooks.use_interval_deps(
        Duration::from_secs_f32(0.5),
        true,
        filter_entities.clone(),
        {
            let world = world.clone();
            let filter_entities = filter_entities.clone();
            move |_| {
                world.get_entities(parent, set_entities.clone(), filter_entities.clone());
            }
        },
    );
    let n_entities = entities.len();
    let entity_list = FlowColumn::el(
        entities
            .into_iter()
            .take(if show_all { usize::MAX } else { MAX })
            .map(move |e| {
                EntityBlock {
                    world: world.clone(),
                    entity: e.clone(),
                    filter_entities: filter_entities.clone(),
                    filter_components: filter_components.clone(),
                }
                .el()
                .memoize_subtree(format!(
                    "{}-{}-{}",
                    e.id, filter_entities, filter_components
                ))
            })
            .collect_vec(),
    );
    if n_entities < MAX || show_all {
        entity_list
    } else {
        FlowColumn::el([
            entity_list,
            Button::new(format!("{} hidden. See all", n_entities - MAX), move |_| {
                set_show_all(true)
            })
            .el(),
        ])
    }
}

#[element_component]
fn EntityBlock(
    hooks: &mut Hooks,
    world: Arc<dyn InspectableWorld>,
    entity: InspectedEntity,
    filter_entities: String,
    filter_components: String,
) -> Element {
    let (expanded, set_expanded) = hooks.use_state(false);
    let (components, set_components) = hooks.use_state(false);
    FlowColumn::el([
        FlowRow::el([
            Button::new(
                if expanded {
                    CHEVRON_DOWN
                } else {
                    CHEVRON_RIGHT
                },
                move |_| set_expanded(!expanded),
            )
            .style(ButtonStyle::Flat)
            .el(),
            Button::new(
                if let Some(name) = &entity.name {
                    name.to_string()
                } else {
                    entity.id.to_string()
                },
                move |_| set_components(!components),
            )
            .style(ButtonStyle::Flat)
            .toggled(components)
            .el(),
        ]),
        if components {
            EntityComponents {
                world: world.clone(),
                entity: entity.id,
                filter_components: filter_components.clone(),
            }
            .el()
            .memoize_subtree(format!("{}-{}", entity.id, filter_components))
        } else {
            Element::new()
        },
        if expanded {
            EntityList {
                world,
                parent: Some(entity.id),
                filter_entities: filter_entities.clone(),
                filter_components: filter_components.clone(),
            }
            .el()
            .with(margin(), Borders::left(STREET).into())
        } else {
            Element::new()
        },
    ])
}

#[element_component]
fn EntityComponents(
    hooks: &mut Hooks,
    world: Arc<dyn InspectableWorld>,
    entity: EntityId,
    filter_components: String,
) -> Element {
    let (components, set_components) = hooks.use_state(Vec::new());
    hooks.use_interval_deps(
        Duration::from_secs_f32(0.5),
        true,
        filter_components.clone(),
        {
            let world = world.clone();
            let filter_components = filter_components.clone();
            move |_| {
                world.get_components(entity, set_components.clone(), filter_components.clone());
            }
        },
    );
    FlowColumn::el(
        components
            .into_iter()
            .enumerate()
            .map(|(i, e)| {
                ComponentBlock {
                    component: e.clone(),
                    odd: i % 2 == 0,
                }
                .el()
                .memoize_subtree(format!("{:?}", e))
            })
            .collect_vec(),
    )
}

#[element_component]
fn ComponentBlock(_hooks: &mut Hooks, component: InspectedComponent, odd: bool) -> Element {
    let inner = FlowRow::el([
        FlowRow::el([Text::el(component.name)
            .with(color(), vec4(1., 1., 1., 1.))
            .with(max_width(), 250.)])
        .with(fit_horizontal(), ambient_layout::Fit::None)
        .with(width(), 260.),
        FlowRow::el([Text::el(component.value).with(max_width(), 300.)])
            .with(fit_horizontal(), ambient_layout::Fit::None)
            .with(width(), 300.),
    ]);
    if odd {
        inner.with_background(vec4(0.1, 0.1, 0.1, 1.))
    } else {
        inner
    }
}
