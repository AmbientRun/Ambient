use std::sync::Arc;

use itertools::Itertools;
use kiwi_core::{name, runtime, selectable, tags};
use kiwi_ecs::{query, uid, EntityId, EntityUid, World};
use kiwi_ecs_editor::ECSEditor;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_network::{
    client::{game_client, GameClient},
    is_remote_entity, log_network_result,
    rpc::rpc_world_diff,
};
use kiwi_std::{cb, cb_arc, Cb};
use kiwi_ui::{fit_horizontal, space_between_items, Button, ButtonStyle, DialogScreen, Fit, FlowColumn, FlowRow, ScrollArea, STREET};

#[derive(Debug, Clone)]
pub struct EntityBrowser {
    on_select: Cb<dyn Fn(EntityId, EntityUid) + Sync + Send>,
}
impl ElementComponent for EntityBrowser {
    fn render(self: Box<Self>, _: &mut World, hooks: &mut Hooks) -> Element {
        let Self { on_select } = *self;
        let (entities, set_entities) = hooks.use_state(Vec::new());
        let (all_tags, set_all_tags) = hooks.use_state(Vec::new());
        let (selected_tag, set_selected_tag) = hooks.use_state(None);
        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
        hooks.use_spawn(move |_| {
            let state = game_client.game_state.lock();
            let entities = query(uid())
                .incl(is_remote_entity())
                .incl(selectable())
                .iter(&state.world, None)
                .map(|(id, uid)| {
                    (
                        id,
                        uid.clone(),
                        state.world.get_ref(id, name()).cloned().unwrap_or_default(),
                        state.world.get_ref(id, tags()).cloned().unwrap_or_default(),
                    )
                })
                .collect_vec();
            let all_tags = entities.iter().flat_map(|entity| &entity.3).sorted().dedup().cloned().collect_vec();
            set_entities(entities);
            set_all_tags(all_tags);
            Box::new(|_| {})
        });
        FlowColumn::el([
            FlowRow(
                all_tags
                    .into_iter()
                    .map(|tag| {
                        let set_selected_tag = set_selected_tag.clone();
                        Button::new(tag.clone(), {
                            let tag = tag.clone();
                            let selected_tag = selected_tag.clone();
                            move |_| {
                                let tag = tag.clone();
                                if Some(tag.clone()) == selected_tag {
                                    set_selected_tag(None);
                                } else {
                                    set_selected_tag(Some(tag));
                                }
                            }
                        })
                        .style(ButtonStyle::Flat)
                        .toggled(Some(tag) == selected_tag)
                        .el()
                    })
                    .collect(),
            )
            .el()
            .set(space_between_items(), STREET),
            FlowColumn(
                entities
                    .into_iter()
                    .filter(|entity| if let Some(selected_tag) = &selected_tag { entity.2.contains(selected_tag) } else { true })
                    .take(100)
                    .map(move |(entity, uid, name, tags)| {
                        Button::new(format!("{entity} {name} {tags:?}"), closure!(clone on_select, |_| on_select.0(entity, uid.clone())))
                            .el()
                    })
                    .collect_vec(),
            )
            .el()
            .set(space_between_items(), STREET),
        ])
        .set(space_between_items(), STREET)
    }
}

#[derive(Debug, Clone)]
pub struct EntityBrowserScreen {
    pub on_select: Cb<dyn Fn(EntityId, EntityUid) + Sync + Send>,
    pub on_back: Cb<dyn Fn() + Sync + Send>,
}
impl ElementComponent for EntityBrowserScreen {
    fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
        let Self { on_select, on_back } = *self;
        let (advanced, set_advanced) = hooks.use_state(false);
        DialogScreen(
            ScrollArea(
                FlowColumn::el([
                    FlowRow::el([
                        Button::new("Back", {
                            move |_| {
                                on_back();
                            }
                        })
                        .style(ButtonStyle::Primary)
                        .el(),
                        Button::new("Advanced", move |_| set_advanced(!advanced)).toggled(advanced).el(),
                    ])
                    .set(space_between_items(), STREET),
                    if advanced {
                        ECSEditor {
                            get_world: cb({
                                let game_client = world.resource(game_client()).clone();
                                move |run| {
                                    let state = game_client.as_ref().unwrap().game_state.lock();
                                    run(&state.world);
                                }
                            }),
                            on_change: cb(|world, diff| {
                                let client = world.resource(game_client()).clone().unwrap();
                                world.resource(runtime()).spawn(async move {
                                    log_network_result!(client.rpc(rpc_world_diff, diff).await);
                                });
                            }),
                        }
                        .el()
                        .memoize_subtree("")
                    } else {
                        EntityBrowser {
                            on_select: cb_arc(Arc::new(move |id, uid| {
                                on_select(id, uid);
                            })),
                        }
                        .el()
                    },
                ])
                .set(space_between_items(), STREET)
                .set(fit_horizontal(), Fit::Parent),
            )
            .el()
            .set(fit_horizontal(), Fit::Parent),
        )
        .el()
    }
}
