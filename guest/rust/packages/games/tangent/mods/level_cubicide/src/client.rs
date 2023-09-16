use ambient_api::{
    core::{
        rendering::components::{double_sided, fog_density, sun},
        transform::components::{rotation, translation},
    },
    element::{use_frame, use_rerender_signal},
    prelude::*,
};

use packages::tangent_schema::player::components as pc;

mod shared;

const RENDER_LEVEL_BOUNDARIES: bool = true;

#[main]
fn main() {
    // Automatically adjust the density of the fog on a cycle
    query(sun()).each_frame(move |suns| {
        const BASE: f32 = 0.02;
        const AMPLITUDE: f32 = 0.06;
        // How many metres the player can travel before the fog is at its maximum density
        const TRANSITION_LENGTH: f32 = 4.0;

        let Some(local_translation) =
            local_vehicle().and_then(|v| entity::get_component(v, translation()))
        else {
            return;
        };

        let sdf = shared::level(local_translation.xy());

        for (sun_id, _) in suns {
            // If the player is in the level carve-out, the fog should be light.
            // Otherwise, it should be heavy.
            let new_density = BASE + AMPLITUDE * (sdf / TRANSITION_LENGTH).clamp(0.0, 1.0);
            entity::set_component(sun_id, fog_density(), new_density)
        }
    });

    if RENDER_LEVEL_BOUNDARIES {
        LevelBoundaries.el().spawn_interactive();
    }
}

fn local_vehicle() -> Option<EntityId> {
    entity::get_component(player::get_local(), pc::vehicle_ref())
}

#[element_component]
fn LevelBoundaries(hooks: &mut Hooks) -> Element {
    use ambient_api::core::{
        app::components::main_scene,
        rect::components::{line_from, line_to, line_width, rect},
        rendering::components::color,
    };

    let rerender = use_rerender_signal(hooks);
    use_frame(hooks, move |_| rerender());

    let Some(vehicle) = local_vehicle() else {
        return Element::new();
    };
    let translation = entity::get_component(vehicle, translation()).unwrap_or_default();
    let rotation = entity::get_component(vehicle, rotation()).unwrap_or_default();
    let yaw = rotation.to_euler(glam::EulerRot::ZYX).0;

    let mut points = vec![];
    for ang in (-45..=45).step_by(5).map(|v| v as f32) {
        let yaw = (yaw.to_degrees() + ang).to_radians();
        let dir = Quat::from_rotation_z(yaw) * -Vec3::Y;

        let point_eval = |t| translation + dir * t;
        if let Some(t) = root_find(0.0, 100.0, |t| shared::level(point_eval(t).xy())) {
            points.push(point_eval(t).xy());
        }
    }

    Group::el(points.windows(2).flat_map(|p| {
        fn make_line(p0: Vec3, p1: Vec3) -> Element {
            Element::new()
                .init_default(rect())
                .with(main_scene(), ())
                .with(line_from(), p0)
                .with(line_to(), p1)
                .with(line_width(), 0.2)
                .with(color(), vec4(0.8, 0.3, 0.0, 1.0))
                .with(double_sided(), true)
        }

        [
            make_line(p[0].extend(0.0), p[1].extend(0.0)),
            make_line(p[0].extend(8.0), p[1].extend(8.0)),
        ]
    }))
}

fn root_find(mut start: f32, mut end: f32, f: impl Fn(f32) -> f32) -> Option<f32> {
    for _ in 0..10 {
        let mid = (start + end) / 2.0;
        let mid_val = f(mid);

        if mid_val > 0.0 {
            end = mid;
        } else {
            start = mid;
        }
    }

    let mid = (start + end) / 2.0;
    if f(mid) < 0.1 {
        Some(mid)
    } else {
        None
    }
}
