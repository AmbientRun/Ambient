use ambient_core::{name, selectable, tags};
use ambient_ecs::{query, EntityId};
use ambient_ecs_editor::{ECSEditor, InspectableAsyncWorld};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_network::{
    client::{game_client, GameClient},
    is_remote_entity,
};
use ambient_std::{cb, Cb};
use ambient_ui_native::{
    fit_horizontal, space_between_items, Button, ButtonStyle, DialogScreen, Fit, FlowColumn,
    FlowRow, ScrollArea, ScrollAreaSizing, STREET,
};
use itertools::Itertools;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EntityBrowser {
    on_select: Cb<dyn Fn(EntityId) + Sync + Send>,
}
impl ElementComponent for EntityBrowser {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { on_select } = *self;
        let (entities, set_entities) = hooks.use_state(Vec::new());
        let (all_tags, set_all_tags) = hooks.use_state(Vec::new());
        let (selected_tag, set_selected_tag) = hooks.use_state(None);
        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
        hooks.use_spawn(move |_| {
            let state = game_client.game_state.lock();
            let entities = query(selectable())
                .incl(is_remote_entity())
                .iter(&state.world, None)
                .map(|(id, _)| {
                    (
                        id,
                        state.world.get_ref(id, name()).cloned().unwrap_or_default(),
                        state.world.get_ref(id, tags()).cloned().unwrap_or_default(),
                    )
                })
                .collect_vec();
            let all_tags = entities
                .iter()
                .flat_map(|entity| &entity.2)
                .sorted()
                .dedup()
                .cloned()
                .collect_vec();
            set_entities(entities);
            set_all_tags(all_tags);
            |_| {}
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
            .with(space_between_items(), STREET),
            FlowColumn(
                entities
                    .into_iter()
                    .filter(|entity| {
                        if let Some(selected_tag) = &selected_tag {
                            entity.2.contains(selected_tag)
                        } else {
                            true
                        }
                    })
                    .take(100)
                    .map(move |(entity, name, tags)| {
                        Button::new(
                            format!("{entity} {name} {tags:?}"),
                            closure!(clone on_select, |_| on_select.0(entity)),
                        )
                        .el()
                    })
                    .collect_vec(),
            )
            .el()
            .with(space_between_items(), STREET),
        ])
        .with(space_between_items(), STREET)
    }
}

#[derive(Debug, Clone)]
pub struct EntityBrowserScreen {
    pub on_select: Cb<dyn Fn(EntityId) + Sync + Send>,
    pub on_back: Cb<dyn Fn() + Sync + Send>,
}
impl ElementComponent for EntityBrowserScreen {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { on_select, on_back } = *self;
        let (advanced, set_advanced) = hooks.use_state(false);
        DialogScreen(
            ScrollArea::el(
                ScrollAreaSizing::FitChildrenWidth,
                FlowColumn::el([
                    FlowRow::el([
                        Button::new("Back", {
                            move |_| {
                                on_back();
                            }
                        })
                        .style(ButtonStyle::Primary)
                        .el(),
                        Button::new("Advanced", move |_| set_advanced(!advanced))
                            .toggled(advanced)
                            .el(),
                    ])
                    .with(space_between_items(), STREET),
                    if advanced {
                        ECSEditor::el(Arc::new(InspectableAsyncWorld(cb({
                            let game_client = hooks.world.resource(game_client()).clone();
                            move |cb| {
                                let state = game_client.as_ref().unwrap().game_state.lock();
                                cb(&state.world);
                            }
                        }))))
                        .memoize_subtree("")
                    } else {
                        EntityBrowser {
                            on_select: cb(move |id| {
                                on_select(id);
                            }),
                        }
                        .el()
                    },
                ])
                .with(space_between_items(), STREET)
                .with(fit_horizontal(), Fit::Parent),
            )
            .with(fit_horizontal(), Fit::Parent),
        )
        .el()
    }
}
