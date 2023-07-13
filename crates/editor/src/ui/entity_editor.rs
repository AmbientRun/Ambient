use std::{sync::Arc, time::Duration};

use ambient_core::{
    name, runtime, snap_to_ground, tags,
    transform::{scale, translation},
};
use ambient_ecs::{
    generated::components::core::animation::animation_errors, with_component_registry, Component,
    ComponentDesc, ComponentEntry, ComponentValue, Entity, EntityId, PrimitiveComponentType, World,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_intent::client_push_intent;
use ambient_network::{client::ClientState, hooks::use_remote_component};
use ambient_physics::collider::{character_controller_height, character_controller_radius, mass};
use ambient_std::{cb, Cb};
use ambient_ui_native::{
    align_horizontal_impl, align_vertical_impl,
    layout::{fit_horizontal_impl, margin, Borders, Fit},
    space_between_items, Align, Button, ButtonStyle, DropdownSelect, Editor, EditorPrompt,
    FlowColumn, FlowRow, ScreenContainer, StylesExt, Text, STREET,
};
use glam::{Vec2, Vec3, Vec4};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::EditingEntityContext;
use crate::intents::intent_component_change;

#[tracing::instrument(level = "info", skip_all)]
#[element_component]
pub fn EntityEditor(hooks: &mut Hooks, entity_id: EntityId) -> Element {
    // tracing::info!("Drawing EntityEditor");
    let (entity, set_entity) = hooks.use_state(None);
    hooks.provide_context(|| EditingEntityContext(entity_id));
    let (client_state, _) = hooks.consume_context::<ClientState>().unwrap();

    hooks.use_interval_deps(
        Duration::from_millis(100),
        false,
        entity_id,
        closure!(clone set_entity, clone client_state, |&entity_id| {
            ambient_profiling::scope!("EntityEditor::update_entity_data");
            let client_state = client_state.game_state.lock();
            set_entity(client_state.world.clone_entity(entity_id).ok());
        }),
    );

    let name =
        use_remote_component(hooks, entity_id, name()).unwrap_or(format!("Entity {entity_id}"));
    let runtime = hooks.world.resource(runtime()).clone();

    if let Some(entity) = entity {
        let _translation = entity.get_cloned(translation());
        FlowColumn(vec![
            Text::el(name).section_style(),
            if let Some(mass) = entity.get(mass()) {
                Text::el(format!("{mass} kg")).small_style()
            } else {
                Element::new()
            },
            EntityComponentsEditor {
                value: entity,
                on_change: cb(move |change| {
                    runtime.spawn(client_push_intent(
                        client_state.clone(),
                        intent_component_change(),
                        (entity_id, change),
                        None,
                        None,
                    ));
                }),
            }
            .el()
            .with(fit_horizontal_impl(), Fit::Parent),
            // if let Some(translation) = translation {
            //     Button::new("Teleport to entity", {
            //         move |_| {
            //             let client_state = client_state.clone();
            //             runtime.spawn(async move {
            //                 client_state.rpc(rpc_teleport_player, translation).await.ok();
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
        .with(space_between_items(), STREET)
    } else {
        Text::el("No such entity")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityComponentChange {
    Change(ComponentEntry),
    Add(ComponentEntry),
    Remove(ComponentDesc),
}
impl EntityComponentChange {
    /// Returns a EntityComponentChange which can be used to revert this change
    pub fn apply_to_entity(&self, world: &mut World, id: EntityId) -> EntityComponentChange {
        match self {
            EntityComponentChange::Change(entry) => {
                EntityComponentChange::Change(world.set_entry(id, entry.clone()).unwrap())
            }
            EntityComponentChange::Add(entry) => {
                world.add_entry(id, entry.clone()).unwrap();
                EntityComponentChange::Remove(entry.desc())
            }
            EntityComponentChange::Remove(desc) => {
                let old = world.get_entry(id, *desc).unwrap();
                world.remove_component(id, *desc).unwrap();
                EntityComponentChange::Add(old)
            }
        }
    }
    pub fn apply_to_entity_data(self, entity: &mut Entity) {
        match self {
            EntityComponentChange::Change(entry) => entity.set_entry(entry),
            EntityComponentChange::Add(entry) => entity.set_entry(entry),
            EntityComponentChange::Remove(desc) => {
                entity.remove_raw(desc);
            }
        }
    }
}

#[tracing::instrument(level = "info", skip_all)]
#[ambient_profiling::function]
#[element_component]
fn EntityComponentsEditor(
    _hooks: &mut Hooks,
    value: Entity,
    on_change: Cb<dyn Fn(EntityComponentChange) + Sync + Send>,
) -> Element {
    let mut missing_components = Vec::new();
    fn reg_component<
        T: ComponentValue + Editor + std::fmt::Debug + Clone + Sync + Send + 'static,
    >(
        entity: &Entity,
        on_change: Cb<dyn Fn(EntityComponentChange) + Sync + Send>,
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
                    on_change: cb(closure!(clone on_change, |value| on_change(EntityComponentChange::Change(value)))),
                    on_remove: cb(move || on_change(EntityComponentChange::Remove(component.into()))),
                }
                .el(),
            ))
        } else {
            missing_components.push((
                display_name.to_string(),
                Arc::new(move || {
                    on_change(EntityComponentChange::Add(ComponentEntry::new(
                        component,
                        on_create(),
                    )))
                }),
            ));
            None
        }
    }

    macro_rules! reg_default_component {
        ($name:expr, $short:expr, $component:expr) => {
            reg_component(
                &value,
                on_change.clone(),
                &mut missing_components,
                $name,
                $short,
                $component,
                Default::default,
            )
        };
    }

    let mut component_editors = [
        reg_default_component!("Name", true, name()),
        reg_default_component!("Tags", false, tags()),
        reg_default_component!("Translation", true, translation()),
        reg_default_component!("Scale", true, scale()),
        // reg_default_component!("Model", false, model_def()),
        // reg_default_component!("Decal", false, decal()),
        reg_default_component!(
            "Character collider radius",
            false,
            character_controller_radius()
        ),
        reg_default_component!(
            "Character collider height",
            false,
            character_controller_height()
        ),
        // reg_default_component!("Collider", false, collider()),
        // reg_default_component!("Collider type", true, collider_type()),
        reg_default_component!("Mass", true, mass()),
        reg_default_component!("Audio Emitter", false, ambient_world_audio::audio_emitter()),
        // reg_default_component!("Loop animation", true, loop_animation()),
        // reg_default_component!("Animation retargeting", true, animation_retargeting()),
        reg_default_component!("Snap to ground", true, snap_to_ground()),
    ]
    .into_iter()
    .flatten()
    .collect_vec();

    with_component_registry(|cr| {
        ambient_profiling::scope!("setup_component_editors");
        fn register_dynamic_component<
            T: ComponentValue + Editor + std::fmt::Debug + Clone + Sync + Send + Default + 'static,
        >(
            (entity, on_change, missing_components): (
                &Entity,
                Cb<dyn Fn(EntityComponentChange) + Sync + Send>,
                &mut Vec<(String, Arc<dyn Fn() + Sync + Send>)>,
            ),
            display_name: &str,
            desc: ComponentDesc,
        ) -> Option<(String, Element)> {
            reg_component(
                entity,
                on_change,
                missing_components,
                display_name,
                true,
                Component::<T>::new(desc),
                Default::default,
            )
        }

        for (comp, desc) in cr.all_external() {
            let display_name = desc.name().unwrap_or_else(|| desc.path());

            let t = (&value, on_change.clone(), &mut missing_components);

            let element = match comp.ty {
                PrimitiveComponentType::Empty => {
                    register_dynamic_component::<()>(t, &display_name, desc)
                }
                PrimitiveComponentType::Bool => {
                    register_dynamic_component::<bool>(t, &display_name, desc)
                }
                // ExternalEcsComponent::EntityId => register_dynamic_component(t, &display_name, desc),
                PrimitiveComponentType::F32 => {
                    register_dynamic_component::<f32>(t, &display_name, desc)
                }
                // ExternalEcsComponent::F64 => register_dynamic_component(t, &display_name, desc),
                // ExternalEcsComponent::Mat4 => register_dynamic_component(t, &display_name, desc),
                PrimitiveComponentType::I32 => {
                    register_dynamic_component::<i32>(t, &display_name, desc)
                }
                // ExternalEcsComponent::Quat => register_dynamic_component(t, &display_name, desc),
                PrimitiveComponentType::String => {
                    register_dynamic_component::<String>(t, &display_name, desc)
                }
                PrimitiveComponentType::U32 => {
                    register_dynamic_component::<u32>(t, &display_name, desc)
                }
                PrimitiveComponentType::U64 => {
                    register_dynamic_component::<u64>(t, &display_name, desc)
                }
                PrimitiveComponentType::Vec2 => {
                    register_dynamic_component::<Vec2>(t, &display_name, desc)
                }
                PrimitiveComponentType::Vec3 => {
                    register_dynamic_component::<Vec3>(t, &display_name, desc)
                }
                PrimitiveComponentType::Vec4 => {
                    register_dynamic_component::<Vec4>(t, &display_name, desc)
                }
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
                    let items = missing_components
                        .iter()
                        .map(|x| Text::el(x.0.to_string()))
                        .collect_vec();
                    DropdownSelect {
                        content: Text::el("Add component"),
                        on_select: cb(move |index| missing_components[index].1()),
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
                        FlowRow::el([Text::el(format!(
                            "Animation errors:\n{}",
                            anim_error
                                .iter()
                                .map(|x| x.split(": ").join(":\n"))
                                .join("\n")
                        ))
                        .error_text_style()]),
                        move |_| {
                            arboard::Clipboard::new()
                                .unwrap()
                                .set_text(anim_error.join("\n"))
                                .ok();
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
    .with(space_between_items(), STREET)
}

#[ambient_profiling::function]
#[element_component]
fn ComponentEditor<T: ComponentValue + Editor + std::fmt::Debug + Clone + Sync + Send + 'static>(
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
    .with(margin(), Borders::right(STREET).into());

    FlowRow(vec![
        ScreenContainer(screen).el(),
        remove,
        Text::el(&display_name).with(margin(), Borders::right(STREET).into()),
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
        .with(align_horizontal_impl(), Align::End)
        .with(fit_horizontal_impl(), Fit::Parent),
    ])
    .el()
    .with(align_vertical_impl(), Align::Center)
    .with(fit_horizontal_impl(), Fit::Parent)
}
