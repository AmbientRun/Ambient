use ambient_api::prelude::*;

#[main]
fn main() {
    default_camera::init();
    ground_and_cubes::init();
    let sun = fog_lighting::spawn_sun();
    ui_sliders_for_sun::init(sun);
}

mod default_camera {
    use ambient_api::{
        core::{
            app::components::main_scene,
            camera::{
                components::{aspect_ratio_from_window, fog},
                concepts::make_perspective_infinite_reverse_camera,
            },
            transform::components::{lookat_target, translation},
        },
        prelude::*,
    };
    pub fn init() {
        Entity::new()
            .with_merge(make_perspective_infinite_reverse_camera())
            .with(aspect_ratio_from_window(), EntityId::resources())
            .with(main_scene(), ())
            .with(fog(), ())
            .with(translation(), vec3(0., -5., 3.))
            .with(lookat_target(), vec3(0., 0., 2.))
            .spawn();
    }
}

mod fog_lighting {
    use ambient_api::{
        core::{
            app::components::main_scene,
            rendering::components::{
                fog_color, fog_density, fog_height_falloff, light_diffuse, sky, sun,
            },
            transform::{components::rotation, concepts::make_transformable},
        },
        prelude::*,
    };
    pub fn spawn_sun() -> EntityId {
        let sun = Entity::new()
            .with_merge(make_transformable())
            .with(sun(), 0.0)
            .with(rotation(), Quat::from_rotation_y(-1.))
            .with(main_scene(), ())
            .with(light_diffuse(), Vec3::ONE)
            .with(fog_color(), vec3(1., 1., 1.))
            .with(fog_density(), 0.1)
            .with(fog_height_falloff(), 0.01)
            .spawn();

        Entity::new()
            .with_merge(make_transformable())
            .with(sky(), ())
            .spawn();

        sun
    }
}

mod ground_and_cubes {
    use ambient_api::{
        core::{
            primitives::components::{cube, quad},
            rendering::components::{cast_shadows, color},
            transform::{
                components::{scale, translation},
                concepts::make_transformable,
            },
        },
        prelude::*,
    };
    pub fn init() {
        Entity::new()
            .with_merge(make_transformable())
            .with(quad(), ())
            .with(scale(), Vec3::ONE * 1000.)
            .with(color(), vec4(1., 0., 0., 1.))
            .spawn();

        for i in 0..10 {
            Entity::new()
                .with_merge(make_transformable())
                .with(cube(), ())
                .with(translation(), vec3(0., 1. * (2f32).powi(i), 1.))
                .with(scale(), Vec3::ONE * 2.)
                .with(color(), vec4(0., 1., 0., 1.))
                .with(cast_shadows(), ())
                .spawn();
        }
    }
}

mod ui_sliders_for_sun {
    use ambient_api::{
        core::rendering::components::{fog_density, fog_height_falloff},
        prelude::*,
    };

    pub fn init(sun: EntityId) {
        App::el(sun).spawn_interactive();
    }
    #[element_component]
    fn App(hooks: &mut Hooks, sun: EntityId) -> Element {
        FocusRoot::el([FlowColumn::el([
            FlowRow::el([
                Text::el("Fog density: "),
                Slider::new_for_entity_component(hooks, sun, fog_density()).el(),
            ]),
            FlowRow::el([
                Text::el("Fog height falloff: "),
                Slider::new_for_entity_component(hooks, sun, fog_height_falloff()).el(),
            ]),
        ])
        .with_background(vec4(0., 0., 0., 0.9))
        .with_padding_even(10.)])
    }
}
