use ambient_api::{
    core::{
        app::components::main_scene,
        rendering::components::color,
        text::components::{font_size, text},
        transform::{
            components::{local_to_parent, mesh_to_local, mesh_to_world},
            concepts::{Transformable, TransformableOptional},
        },
    },
    prelude::*,
};
use packages::tangent_schema::vehicle::components as vc;

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

    query((vc::hud(), vc::speed_kph())).each_frame(|huds| {
        for (_, (hud_id, speed_kph)) in huds {
            entity::set_component(hud_id, text(), format!("{:.1}\n", speed_kph));
        }
    });
}
