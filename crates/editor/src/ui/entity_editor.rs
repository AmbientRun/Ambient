use std::{sync::Arc, time::Duration};

use glam::{Vec2, Vec3, Vec4};
use itertools::Itertools;
use kiwi_animation::{animation_errors, animation_retargeting, loop_animation};
use kiwi_core::{
    name, runtime, snap_to_ground, tags,
    transform::{scale, translation},
};
use kiwi_decals::decal;
use kiwi_ecs::{
    uid, with_component_registry, Component, ComponentDesc, ComponentEntry, ComponentValue, EntityData, EntityId, PrimitiveComponentType,
    World,
};
use kiwi_element::{element_component, Element, ElementComponentExt, Hooks};
use kiwi_intent::client_push_intent;
use kiwi_network::{client::GameClient, hooks::use_remote_component};
use kiwi_physics::collider::{character_controller_height, character_controller_radius, collider, collider_type, mass};
use kiwi_std::{asset_url::ObjectRef, cb, cb_arc, Cb, IntoDuration};
use kiwi_ui::{
    align_horizontal, align_vertical,
    layout::{fit_horizontal, margin, Borders, Fit},
    space_between_items, use_interval_deps, Align, Button, ButtonStyle, DropdownSelect, Editor, EditorPrompt, FlowColumn, FlowRow,
    ScreenContainer, StylesExt, Text, STREET,
};
use serde::{Deserialize, Serialize};
use winit::event::VirtualKeyCode;

use super::EditingEntityContext;
use crate::intents::intent_component_change;

#[tracing::instrument(level = "info", skip_all)]
#[element_component]
pub fn EntityEditor(world: &mut World, hooks: &mut Hooks, entity_id: EntityId) -> Element {
    // tracing::info!("Drawing EntityEditor");
    let (entity, set_entity) = hooks.use_state(None);
    hooks.provide_context(|| EditingEntityContext(entity_id));
    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();

    use_interval_deps(
        world,
        hooks,
        Duration::from_millis(100),
        false,
        entity_id,
        closure!(clone set_entity, clone game_client, |&entity_id| {
            profiling::scope!("EntityEditor::update_entity_data");
            let game_state = game_client.game_state.lock();
            if let Ok(data) = game_state.world.clone_entity(entity_id) {
                set_entity(Some(data));
            } else {
                set_entity(None);
            }
        }),
    );

    let name = use_remote_component(hooks, entity_id, name()).unwrap_or(format!("Entity {entity_id}"));
    let runtime = world.resource(runtime()).clone();

    if let Some(entity) = entity {
        let uid = entity.get_ref(uid()).cloned().unwrap();
        let translation = entity.get_cloned(translation());
        FlowColumn(vec![
            Text::el(name).section_style(),
            if let Some(mass) = entity.get(mass()) { Text::el(format!("{mass} kg")).small_style() } else { Element::new() },
            ObjectComponentsEditor {
                value: entity,
                on_change: cb_arc(Arc::new({
                    let game_client = game_client.clone();
                    let runtime = runtime.clone();
                    move |change| {
                        runtime.spawn(client_push_intent(
                            game_client.clone(),
                            intent_component_change(),
                            (uid.clone(), change),
                            None,
                            None,
                        ));
                    }
                })),
            }
            .el()
            .set(fit_horizontal(), Fit::Parent),
            // if let Some(translation) = translation {
            //     Button::new("Teleport to entity", {
            //         move |_| {
            //             let game_client = game_client.clone();
            //             runtime.spawn(async move {
            //                 game_client.rpc(rpc_teleport_player, translation).await.ok();
            //             });
            //         }
            //     })
            //     .hotkey(VirtualKeyCode::F)
            //     .el()
            // } else {
            //     Element::new()
            // },
        ])
        .el()
        .set(space_between_items(), STREET)
    } else {
        Text::el("No such entity")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectComponentChange {
    Change(ComponentEntry),
    Add(ComponentEntry),
    Remove(ComponentDesc),
}
impl ObjectComponentChange {
    /// Returns a ObjectComponentChange which can be used to revert this change
    pub fn apply_to_entity(&self, world: &mut World, id: EntityId) -> ObjectComponentChange {
        match self {
            ObjectComponentChange::Change(entry) => ObjectComponentChange::Change(world.set_entry(id, entry.clone()).unwrap()),
            ObjectComponentChange::Add(entry) => {
                world.add_entry(id, entry.clone()).unwrap();
                ObjectComponentChange::Remove(entry.desc())
            }
            ObjectComponentChange::Remove(desc) => {
                let old = world.get_entry(id, *desc).unwrap();
                world.remove_component(id, *desc).unwrap();
                ObjectComponentChange::Add(old)
            }
        }
    }
    pub fn apply_to_entity_data(self, entity: &mut EntityData) {
        match self {
            ObjectComponentChange::Change(entry) => entity.set_entry(entry),
            ObjectComponentChange::Add(entry) => entity.set_entry(entry),
            ObjectComponentChange::Remove(desc) => {
                entity.remove_raw(desc);
            }
        }
    }
}

#[tracing::instrument(level = "info", skip_all)]
#[profiling::function]
#[element_component]
fn ObjectComponentsEditor(
    _world: &mut World,
    _hooks: &mut Hooks,
    value: EntityData,
    on_change: Cb<dyn Fn(ObjectComponentChange) + Sync + Send>,
) -> Element {
    let mut missing_components = Vec::new();
    fn reg_component<T: ComponentValue + Editor + std::fmt::Debug + Clone + Sync + Send + 'static>(
        entity: &EntityData,
        on_change: Cb<dyn Fn(ObjectComponentChange) + Sync + Send>,
        missing_components: &mut Vec<(String, Arc<dyn Fn() + Sync + Send>)>,
        display_name: &str,
        short: bool,
        component: Component<T>,
        on_create: impl Fn() -> T + Sync + Send + 'static,
    ) -> Option<(String, Element)> {
        let value = entity.get_ref(component).cloned();
        if let Some(value) = value {
            Some((
                display_name.to_string(),
                ComponentEditor {
                    value,
                    component,
                    display_name: display_name.to_string(),
                    inline: short,
                    on_change: cb_arc(Arc::new(closure!(clone on_change, |value| on_change(ObjectComponentChange::Change(value))))),
                    on_remove: cb_arc(Arc::new(move || on_change(ObjectComponentChange::Remove(component.into())))),
                }
                .el(),
            ))
        } else {
            missing_components.push((
                display_name.to_string(),
                Arc::new(move || on_change(ObjectComponentChange::Add(ComponentEntry::new(component, on_create())))),
            ));
            None
        }
    }

    macro_rules! reg_default_component {
        ($name:expr, $short:expr, $component:expr) => {
            reg_component(&value, on_change.clone(), &mut missing_components, $name, $short, $component, Default::default)
        };
    }

    let mut component_editors = [
        reg_default_component!("Name", true, name()),
        reg_default_component!("Tags", false, tags()),
        reg_default_component!("Translation", true, translation()),
        reg_default_component!("Scale", true, scale()),
        // reg_default_component!("Model", false, model_def()),
        reg_default_component!("Decal", false, decal()),
        reg_default_component!("Character collider radius", false, character_controller_radius()),
        reg_default_component!("Character collider height", false, character_controller_height()),
        reg_default_component!("Collider", false, collider()),
        reg_default_component!("Collider type", true, collider_type()),
        reg_default_component!("Mass", true, mass()),
        reg_default_component!("Audio Emitter", false, kiwi_world_audio::audio_emitter()),
        reg_default_component!("Loop animation", true, loop_animation()),
        reg_default_component!("Animation retargeting", true, animation_retargeting()),
        reg_default_component!("Snap to ground", true, snap_to_ground()),
    ]
    .into_iter()
    .flatten()
    .collect_vec();

    with_component_registry(|cr| {
        profiling::scope!("setup_component_editors");
        fn register_dynamic_component<T: ComponentValue + Editor + std::fmt::Debug + Clone + Sync + Send + Default + 'static>(
            (entity, on_change, missing_components): (
                &EntityData,
                Cb<dyn Fn(ObjectComponentChange) + Sync + Send>,
                &mut Vec<(String, Arc<dyn Fn() + Sync + Send>)>,
            ),
            display_name: &str,
            desc: ComponentDesc,
        ) -> Option<(String, Element)> {
            reg_component(entity, on_change, missing_components, display_name, true, Component::<T>::new(desc), Default::default)
        }

        for (comp, desc) in cr.all_external() {
            let display_name = desc.name().unwrap_or_else(|| desc.path());

            let t = (&value, on_change.clone(), &mut missing_components);

            let element = match comp.ty {
                PrimitiveComponentType::Empty => register_dynamic_component::<()>(t, &display_name, desc),
                PrimitiveComponentType::Bool => register_dynamic_component::<bool>(t, &display_name, desc),
                // ExternalEcsComponent::EntityId => register_dynamic_component(t, &display_name, desc),
                PrimitiveComponentType::F32 => register_dynamic_component::<f32>(t, &display_name, desc),
                // ExternalEcsComponent::F64 => register_dynamic_component(t, &display_name, desc),
                // ExternalEcsComponent::Mat4 => register_dynamic_component(t, &display_name, desc),
                PrimitiveComponentType::I32 => register_dynamic_component::<i32>(t, &display_name, desc),
                // ExternalEcsComponent::Quat => register_dynamic_component(t, &display_name, desc),
                PrimitiveComponentType::String => register_dynamic_component::<String>(t, &display_name, desc),
                PrimitiveComponentType::U32 => register_dynamic_component::<u32>(t, &display_name, desc),
                PrimitiveComponentType::U64 => register_dynamic_component::<u64>(t, &display_name, desc),
                PrimitiveComponentType::Vec2 => register_dynamic_component::<Vec2>(t, &display_name, desc),
                PrimitiveComponentType::Vec3 => register_dynamic_component::<Vec3>(t, &display_name, desc),
                PrimitiveComponentType::Vec4 => register_dynamic_component::<Vec4>(t, &display_name, desc),
                PrimitiveComponentType::ObjectRef => register_dynamic_component::<ObjectRef>(t, &display_name, desc),
                _ => None,
            };

            if let Some(element) = element {
                component_editors.push(element);
            }
        }
    });

    component_editors.sort_by(|x, y| x.0.cmp(&y.0));
    missing_components.sort_by(|x, y| x.0.cmp(&y.0));

    FlowColumn::el(
        component_editors
            .into_iter()
            .map(|x| x.1)
            .chain([
                if !missing_components.is_empty() {
                    let items = missing_components.iter().map(|x| Text::el(x.0.to_string())).collect_vec();
                    DropdownSelect {
                        content: Text::el("Add component"),
                        on_select: cb_arc(Arc::new(move |index| missing_components[index].1())),
                        items,
                        inline: false,
                    }
                    .el()
                } else {
                    Element::new()
                },
                if let Some(anim_error) = value.get_ref(animation_errors()) {
                    let anim_error = anim_error.clone();
                    Button::new(
                        FlowRow::el([Text::el(format!("Animation errors:\n{}", anim_error.split(": ").join(":\n"))).error_text_style()]),
                        move |_| {
                            arboard::Clipboard::new().unwrap().set_text(anim_error.clone()).ok();
                        },
                    )
                    .style(ButtonStyle::Flat)
                    .el()
                } else {
                    Element::new()
                },
            ])
            .collect_vec(),
    )
    .set(space_between_items(), STREET)
}

#[profiling::function]
#[element_component]
fn ComponentEditor<T: ComponentValue + Editor + std::fmt::Debug + Clone + Sync + Send + 'static>(
    _world: &mut World,
    hooks: &mut Hooks,
    component: Component<T>,
    value: T,
    display_name: String,
    inline: bool,
    on_change: Cb<dyn Fn(ComponentEntry) + Sync + Send>,
    on_remove: Cb<dyn Fn() + Sync + Send>,
) -> Element {
    let (screen, set_screen) = hooks.use_state(None);
    let remove = Button::new("\u{f6bf}", move |_| {
        on_remove();
    })
    .style(ButtonStyle::Flat)
    .tooltip("Delete")
    .el()
    .set(margin(), Borders::right(STREET));

    FlowRow(vec![
        ScreenContainer(screen).el(),
        remove,
        Text::el(&display_name).set(margin(), Borders::right(STREET)),
        FlowRow(vec![if inline {
            T::editor(
                value,
                cb(move |new_value| {
                    on_change(ComponentEntry::new(component, new_value));
                }),
                Default::default(),
            )
        } else {
            Button::new("\u{fb4e} Edit", move |_| {
                set_screen(Some(
                    EditorPrompt {
                        title: display_name.clone(),
                        value: value.clone(),
                        set_screen: set_screen.clone(),
                        on_ok: cb({
                            let on_change = on_change.clone();
                            move |_, new_value| {
                                on_change(ComponentEntry::new(component, new_value));
                            }
                        }),
                        on_cancel: Some(cb(|_| {})),
                        validator: None,
                    }
                    .el(),
                ));
            })
            .style(ButtonStyle::Flat)
            .el()
        }])
        .el()
        .set(align_horizontal(), Align::End)
        .set(fit_horizontal(), Fit::Parent),
    ])
    .el()
    .set(align_vertical(), Align::Center)
    .set(fit_horizontal(), Fit::Parent)
}
