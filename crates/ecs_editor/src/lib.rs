use std::{collections::HashMap, time::Duration};

use ambient_ecs::{with_component_registry, ComponentDesc, Entity, EntityId, Query, World, WorldDiff};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_renderer::color;
use ambient_std::{cb, Cb};
use ambient_ui::{fit_horizontal, space_between_items, Button, ButtonStyle, Fit, FlowColumn, FlowRow, Text, UIExt, STREET};
use glam::{vec4, Vec4};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct ECSEditor {
    pub get_world: Cb<dyn Fn(Cb<dyn Fn(&World) + Sync + Send>) + Sync + Send>,
    pub on_change: Cb<dyn Fn(&mut World, WorldDiff) + Sync + Send>,
}
impl ElementComponent for ECSEditor {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { get_world, on_change } = *self;
        let (components, set_components) = hooks.use_state(HashMap::<ComponentDesc, bool>::new());
        let (entity_datas, set_entity_datas) = hooks.use_state(Vec::new());
        let (entities, set_entities) = hooks.use_state(Vec::new());
        let (edit_filter, set_edit_filter) = hooks.use_state(false);
        hooks.use_interval_deps(Duration::from_millis(500), false, components.clone(), {
            let get_world = get_world.clone();
            move |components| {
                let mut query = Query::all();
                for (&comp, incl) in components {
                    if *incl {
                        query = query.incl_ref(comp);
                    } else {
                        query = query.excl_ref(comp);
                    }
                }
                let set_entity_datas = set_entity_datas.clone();
                let set_entities = set_entities.clone();
                get_world(cb(move |world| {
                    set_entities(query.iter(world, None).map(|ea| ea.id()).collect());
                    set_entity_datas(
                        query.iter(world, None).take(20).map(|ea| (ea.id(), world.clone_entity(ea.id()).unwrap())).collect_vec(),
                    );
                }));
            }
        });

        let render_component = |id: &str, comp: ComponentDesc| {
            let toggled = components.get(&comp).cloned();
            let components = components.clone();
            let set_components = set_components.clone();
            let on_change = on_change.clone();
            let entities = entities.clone();
            FlowRow::el([
                Button::new("\u{f6bf}", {
                    move |world| {
                        let entities = entities.clone();
                        let mut diff = WorldDiff::new();
                        for id in &entities {
                            diff = diff.remove_component(*id, comp);
                        }
                        on_change(world, diff);
                    }
                })
                .style(ButtonStyle::Flat)
                .tooltip("Delete component from all selected entities")
                .el(),
                Text::el(id)
                    .with(
                        color(),
                        match toggled {
                            Some(true) => vec4(0., 1., 0., 1.),
                            Some(false) => vec4(1., 0., 0., 1.),
                            None => Vec4::ONE,
                        },
                    )
                    .with_clickarea()
                    .on_mouse_up(move |_, _, _| {
                        let mut comps = components.clone();
                        if let Some(v) = comps.get(&comp).cloned() {
                            if v {
                                comps.insert(comp, false);
                            } else {
                                comps.remove(&comp);
                            }
                        } else {
                            comps.insert(comp, true);
                        }
                        set_components(comps);
                    })
                    .el(),
            ])
            .with(space_between_items(), 5.)
        };
        FlowColumn::el([
            Button::new("Filter", move |_| set_edit_filter(!edit_filter)).toggled(edit_filter).el(),
            if edit_filter {
                FlowRow::el(with_component_registry(|r| {
                    r.all()
                        .map(|c| (c.path(), c))
                        .sorted_by_key(|(id, _)| id.to_string())
                        .map(|(path, comp)| render_component(&path, comp))
                        .collect_vec()
                }))
                .with(fit_horizontal(), Fit::Parent)
                .with(space_between_items(), STREET)
            } else {
                Element::new()
            },
            FlowColumn::el([
                Text::el(format!("{} entities selected", entities.len())),
                FlowColumn::el(
                    entity_datas.into_iter().map(|(id, data)| EntityEditor { id, data, on_change: on_change.clone() }.el()).collect_vec(),
                ),
            ]),
        ])
        .with(fit_horizontal(), Fit::Parent)
    }
}

#[derive(Debug, Clone)]
struct EntityEditor {
    id: EntityId,
    data: Entity,
    on_change: Cb<dyn Fn(&mut World, WorldDiff) + Sync + Send>,
}

impl ElementComponent for EntityEditor {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        let Self { id, data, on_change } = *self;

        FlowRow::el([
            FlowColumn::el([
                Text::el(id.to_string()),
                Button::new("\u{f6bf}", move |world| on_change(world, WorldDiff::new().despawn(vec![id]))).style(ButtonStyle::Flat).el(),
            ]),
            FlowColumn::el(
                data.iter()
                    .map(|entry| {
                        FlowRow::el([
                            Text::el(format!("{}:", entry.desc().path())).with(color(), vec4(1., 1., 0., 1.)),
                            Text::el(ellipsis_text(format!("{:?}", entry.as_debug()))),
                        ])
                        .with(space_between_items(), STREET)
                    })
                    .collect_vec(),
            ),
        ])
        .with(space_between_items(), STREET)
    }
}

fn ellipsis_text(text: String) -> String {
    if text.len() > 30 {
        format!("{}...", &text[0..30])
    } else {
        text
    }
}
