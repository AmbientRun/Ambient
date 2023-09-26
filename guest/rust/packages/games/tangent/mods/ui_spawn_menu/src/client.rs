use std::fmt::Display;

use ambient_api::{
    core::{
        rect::components::background_color,
        text::{components::font_style, types::FontStyle},
        ui::components::focusable,
    },
    element::{use_entity_component, use_query, use_state},
    prelude::*,
    ui::use_keyboard_input,
};
use packages::{
    tangent_schema::{concepts::VehicleDef, player::components as pc},
    this::messages::VehicleSpawnRequest,
};

#[main]
pub fn main() {
    App {}.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let in_vehicle = use_entity_component(hooks, player::get_local(), pc::vehicle_ref()).is_some();

    let (toggle, set_toggle) = use_state(hooks, false);
    use_keyboard_input(hooks, {
        let set_toggle = set_toggle.clone();
        move |_, keycode, modifiers, pressed| {
            if in_vehicle {
                return;
            }

            if modifiers == ModifiersState::empty()
                && keycode == Some(VirtualKeyCode::Q)
                && !pressed
            {
                set_toggle(!toggle);
            }
        }
    });

    if toggle {
        SpawnMenu::el(cb(move || set_toggle(false)))
    } else {
        Element::new()
    }
}

#[element_component]
pub fn SpawnMenu(hooks: &mut Hooks, hide: Cb<dyn Fn() + Send + Sync>) -> Element {
    let mut defs = use_query(hooks, VehicleDef::as_query());
    defs.sort_by_key(|(_, c)| c.name.clone());

    WindowSized::el([with_rect(
        FlowColumn::el([
            Text::el("Spawn Menu").header_style(),
            FlowColumn::el(defs.into_iter().map(|(id, def)| {
                SpawnEntry::el(
                    id,
                    def,
                    cb({
                        let hide = hide.clone();
                        move |def_id| {
                            VehicleSpawnRequest { def_id }.send_server_reliable();
                            hide();
                        }
                    }),
                )
            }))
            .with(space_between_items(), 4.0)
            .with(fit_horizontal(), Fit::Parent),
        ])
        .with(space_between_items(), 4.0)
        .with(fit_horizontal(), Fit::Parent),
    )
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))
    .with_padding_even(STREET)])
    .with_padding_even(20.)
    .with_clickarea()
    .el()
    .with(focusable(), "TangentSpawnMenu".to_string())
}

#[element_component]
pub fn SpawnEntry(
    _hooks: &mut Hooks,
    def_id: EntityId,
    def: VehicleDef,
    spawn_def_id: Cb<dyn Fn(EntityId) + Send + Sync>,
) -> Element {
    let stats: &[(&str, &dyn Display)] = &[
        ("Health", &def.max_health),
        ("Altitude", &format!("{}m", def.target)),
        ("Forward Force", &def.forward_force),
    ];

    with_rect(
        FlowRow::el([
            // Contents
            FlowColumn::el([
                // Header
                Text::el(def.name).with(font_style(), FontStyle::Bold),
                // Description
                Text::el(
                    stats
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join(" | "),
                ),
            ])
            .with(space_between_items(), 4.0)
            .with(align_vertical(), Align::Center),
        ])
        .with_padding_even(8.0)
        .with(space_between_items(), 8.0),
    )
    .with(fit_horizontal(), Fit::Parent)
    .with(background_color(), vec4(0., 0., 0., 0.5))
    .with_clickarea()
    .on_mouse_up(move |_, _, button| {
        if button == MouseButton::Left {
            spawn_def_id(def_id);
        }
    })
    .el()
}
