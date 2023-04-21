use std::sync::Arc;

use ambient_core::name;
use ambient_ecs::{query, EntityId, World};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_renderer::color;
use ambient_std::{cb, Cb};
use ambient_ui_native::{margin, Borders, Button, ButtonStyle, FlowColumn, FlowRow, Text, MOVE_DOWN_ICON, MOVE_UP_ICON, STREET};
use glam::vec4;
use itertools::Itertools;

pub trait InspectableWorld: Sync + Send + std::fmt::Debug {
    fn get_entities(&self, parent: Option<EntityId>, cb: Cb<dyn Fn(Vec<InspectedEntity>) + Sync + Send>);
    fn get_components(&self, entity: EntityId, cb: Cb<dyn Fn(Vec<InspectedComponent>) + Sync + Send>);
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
    fn get_entities(&self, parent: Option<ambient_ecs::EntityId>, callback: ambient_std::Cb<dyn Fn(Vec<InspectedEntity>) + Sync + Send>) {
        (self.0)(cb(move |world| {
            let entities = if let Some(parent) = parent {
                query(ambient_core::hierarchy::parent())
                    .collect_cloned(world, None)
                    .into_iter()
                    .filter_map(|(id, this_parent)| {
                        if this_parent == parent {
                            Some(InspectedEntity { id, name: world.get_ref(id, name()).map(|x| x.clone()).ok() })
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
                    .map(|(id, _)| InspectedEntity { id, name: world.get_ref(id, name()).map(|x| x.clone()).ok() })
                    .collect_vec()
            };
            callback(entities);
        }));
    }

    fn get_components(&self, entity: EntityId, callback: Cb<dyn Fn(Vec<InspectedComponent>) + Sync + Send>) {
        (self.0)(cb(move |world: &World| {
            let comps = if let Ok(comps) = world.get_components(entity) {
                comps
                    .into_iter()
                    .map(|comp| InspectedComponent {
                        name: comp.path(),
                        value: format!("{:?}", world.get_entry(entity, comp).unwrap().as_debug()),
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
pub fn ECSEditor(_hooks: &mut Hooks, world: Arc<dyn InspectableWorld>) -> Element {
    EntityList { world, parent: None }.el()
}

#[element_component]
fn EntityList(hooks: &mut Hooks, world: Arc<dyn InspectableWorld>, parent: Option<EntityId>) -> Element {
    let (show_all, set_show_all) = hooks.use_state(false);
    let (entities, set_entities) = hooks.use_state(Vec::new());
    const MAX: usize = 30;
    hooks.use_interval(0.5, {
        let world = world.clone();
        move || {
            world.get_entities(parent, set_entities.clone());
        }
    });
    if entities.len() < MAX || show_all {
        FlowColumn::el(entities.into_iter().map(|e| EntityBlock { world: world.clone(), entity: e }.el()).collect_vec())
    } else {
        let hidden = entities.len() - MAX;
        FlowColumn::el([
            FlowColumn::el(entities.into_iter().take(MAX).map(|e| EntityBlock { world: world.clone(), entity: e }.el()).collect_vec()),
            Button::new(format!("{} hidden. See all", hidden), move |_| set_show_all(true)).el(),
        ])
    }
}

#[element_component]
fn EntityBlock(hooks: &mut Hooks, world: Arc<dyn InspectableWorld>, entity: InspectedEntity) -> Element {
    let (expanded, set_expanded) = hooks.use_state(false);
    let (components, set_components) = hooks.use_state(false);
    FlowColumn::el([
        FlowRow::el([
            Button::new(if expanded { MOVE_DOWN_ICON } else { MOVE_UP_ICON }, move |_| set_expanded(!expanded))
                .style(ButtonStyle::Flat)
                .el(),
            Button::new(if let Some(name) = &entity.name { name.to_string() } else { entity.id.to_string() }, move |_| {
                set_components(!components)
            })
            .style(ButtonStyle::Flat)
            .toggled(components)
            .el(),
        ]),
        if components { EntityComponents { world: world.clone(), entity: entity.id }.el() } else { Element::new() },
        if expanded { EntityList { world, parent: Some(entity.id) }.el().with(margin(), Borders::left(STREET)) } else { Element::new() },
    ])
}

#[element_component]
fn EntityComponents(hooks: &mut Hooks, world: Arc<dyn InspectableWorld>, entity: EntityId) -> Element {
    let (components, set_components) = hooks.use_state(Vec::new());
    hooks.use_interval(0.5, {
        let world = world.clone();
        move || {
            world.get_components(entity, set_components.clone());
        }
    });
    FlowColumn::el(components.into_iter().map(|e| ComponentBlock { component: e }.el()).collect_vec())
}

#[element_component]
fn ComponentBlock(_hooks: &mut Hooks, component: InspectedComponent) -> Element {
    FlowRow::el([Text::el(format!("{}: ", component.name)).with(color(), vec4(1., 1., 0., 1.)), Text::el(component.value)])
}
