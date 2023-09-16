use ambient_api::{
    core::{
        app::components::{main_scene, name},
        physics::components::linear_velocity,
        rendering::components::color,
        text::components::{font_size, text},
        transform::{
            components::{local_to_parent, mesh_to_local, mesh_to_world, rotation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    prelude::*,
};
use packages::tangent_schema::{messages::Input, vehicle::components as vc};

#[main]
pub fn main() {
    spawn_query(vc::player_ref()).bind(move |vehicles| {
        for (id, _) in vehicles {
            let hud_id = Transformable {
                local_to_world: default(),
                optional: TransformableOptional {
                    translation: Some(vec3(0.35, 0., 0.3)),
                    rotation: Some(
                        Quat::from_rotation_z(25.0f32.to_radians())
                            * Quat::from_rotation_x(-65.0f32.to_radians()),
                    ),
                    scale: Some(Vec3::ONE * 0.005),
                },
            }
            .make()
            .with(local_to_parent(), Default::default())
            .with(mesh_to_local(), Default::default())
            .with(mesh_to_world(), Default::default())
            .with(main_scene(), ())
            .with(text(), "0".to_string())
            .with(color(), vec4(1., 1., 1., 1.))
            .with(font_size(), 48.0)
            .spawn();

            entity::add_component(id, vc::hud(), hud_id);
            entity::add_child(id, hud_id);
        }
    });

    despawn_query(vc::hud())
        .requires(vc::player_ref())
        .bind(move |vehicles| {
            for (_vehicle_id, hud_id) in vehicles {
                entity::despawn(hud_id);
            }
        });

    // HACK: despawn all wheels on spawn
    spawn_query(name()).bind(|entities| {
        for (id, name) in entities {
            if name.starts_with("wheel") {
                entity::despawn(id);
            }
        }
    });

    query((rotation(), linear_velocity()))
        .requires(vc::player_ref())
        .each_frame(|vehicles| {
            for (id, (rot, lv)) in vehicles {
                entity::add_component(id, vc::speed_kph(), lv.dot(rot * -Vec3::Y) * 3.6);
            }
        });

    query((vc::hud(), vc::speed_kph())).each_frame(|huds| {
        for (_, (hud_id, speed_kph)) in huds {
            entity::set_component(hud_id, text(), format!("{:.1}\n", speed_kph));
        }
    });

    fixed_rate_tick(Duration::from_millis(20), |_| {
        if !input::is_game_focused() {
            return;
        }

        let input = input::get();
        let direction = {
            let mut direction = Vec2::ZERO;
            if input.keys.contains(&KeyCode::W) {
                direction.y += 1.;
            }
            if input.keys.contains(&KeyCode::S) {
                direction.y -= 1.;
            }
            if input.keys.contains(&KeyCode::A) {
                direction.x -= 1.;
            }
            if input.keys.contains(&KeyCode::D) {
                direction.x += 1.;
            }
            direction
        };
        Input {
            direction,
            jump: input.keys.contains(&KeyCode::Space),
        }
        .send_server_unreliable();
    });
}
