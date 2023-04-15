use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::local_user_id,
        transform::{lookat_center, rotation, translation},
    },
    concepts::make_perspective_infinite_reverse_camera,
    messages::Frame,
    player::KeyCode,
    prelude::*,
};
use ambient_ui_components::prelude::*;
use components::player_vehicle;

#[main]
pub fn main() {
    let camera_id = Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(5., 5., 2.))
        .with(lookat_center(), vec3(0., 0., 1.))
        .spawn();

    Frame::subscribe(move |_| {
        const CAMERA_OFFSET: Vec3 = vec3(-1., 2.5, 0.5);

        let player_id = local_entity_id();
        let Some(vehicle_id) = entity::get_component(player_id, player_vehicle()) else { return; };
        let Some(vehicle_position) = entity::get_component(vehicle_id, translation()) else { return; };
        let Some(vehicle_rotation) = entity::get_component(vehicle_id, rotation()) else { return; };

        let camera_position = vehicle_position + vehicle_rotation * CAMERA_OFFSET;
        entity::set_component(camera_id, translation(), camera_position);
        entity::set_component(
            camera_id,
            lookat_center(),
            camera_position + vehicle_rotation * -Vec3::Y,
        );

        let (delta, input) = player::get_raw_input_delta();
        messages::Input::new(
            input.keys.contains(&KeyCode::Space),
            input.keys.contains(&KeyCode::R),
        )
        .send_server_unreliable();
    });

    DebugUI.el().spawn_interactive();
    DebugLines.el().spawn_interactive();
}

#[element_component]
fn DebugUI(hooks: &mut Hooks) -> Element {
    let messages = hooks.use_query(components::debug_messages());

    FlowColumn::el(messages.into_iter().map(|(id, msgs)| {
        FlowColumn::el([
            Text::el(format!("{}", id)).section_style(),
            FlowColumn::el(
                msgs.into_iter()
                    .map(|s| Text::el(s).with(color(), vec4(1., 1., 1., 1.))),
            ),
        ])
    }))
    .with_padding_even(10.)
    .with_background(vec4(1., 1., 1., 0.02))
}

#[element_component]
fn DebugLines(hooks: &mut Hooks) -> Element {
    let lines = hooks.use_query(components::debug_lines());

    Group::el(lines.into_iter().flat_map(|(_, lines)| {
        lines
            .chunks(2)
            .map(|line| {
                let [start, end]: [Vec3; 2] = line.try_into().unwrap();

                Element::new()
                    .init_default(rect())
                    .with_default(main_scene())
                    .with(line_from(), start)
                    .with(line_to(), end)
                    .with(line_width(), 0.05)
                    .with(color(), vec4(1., 1., 1., 1.))
            })
            .collect::<Vec<_>>()
    }))
}

// TODO: add to API
fn local_entity_id() -> EntityId {
    player::get_by_user_id(&entity::get_component(entity::resources(), local_user_id()).unwrap())
        .unwrap()
}
