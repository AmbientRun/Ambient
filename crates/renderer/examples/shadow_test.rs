use ambient_app::{App, AppBuilder};
use ambient_core::{
    asset_cache,
    bounding::world_bounding_sphere,
    camera::{active_camera, far, near, Camera, Projection},
    main_scene,
    transform::*,
};
use ambient_ecs::{query, FnSystem, Resource, World};
use ambient_element::ElementComponentExt;
use ambient_gizmos::{gizmos, GizmoPrimitive};
use ambient_primitives::Cube;
use ambient_renderer::{cast_shadows, color, RendererConfig};
use ambient_std::{line_hash, math::SphericalCoords};
use env_logger::Env;
use glam::*;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};

async fn init(app: &mut App) {
    let world = &mut app.world;

    let _assets = world.resource(asset_cache()).clone();

    let size = 2000.;

    Cube.el().with(scale(), vec3(size, size, 1.)).with_default(cast_shadows()).spawn_static(world);

    for s in 1..5 {
        let scale_ = (2f32).powi(s);
        for y in 0..5 {
            for x in 0..5 {
                Cube.el()
                    .with(translation(), (scale_ * 20. * (-1. + 2. * vec2(x as f32, y as f32) / 5.)).extend(0.))
                    .with(scale(), vec3(1., 1., 10.))
                    .with(color(), vec4(0.7, 0.7, 0.7, 1.))
                    .with_default(cast_shadows())
                    .spawn_static(world);
            }
        }
    }

    for x in 0..5 {
        let p = (2f32).powi(x);
        Cube.el()
            .with(translation(), 100. + 10. * vec3(p, 0., 0.))
            .with(scale(), Vec3::ONE * p)
            .with(color(), vec4(0.7, 0.7, 0.7, 1.))
            .with_default(cast_shadows())
            .spawn_static(world);
    }

    let sun_direction = vec3(0., 1., 1.).normalize();
    let demo_cam = {
        let view = Mat4::look_at_lh(vec3(-30., 0., 5.), Vec3::ZERO, Vec3::Z);
        Camera { projection: Projection::Perspective { near: 0.1, far: 100., fovy: 1., aspect_ratio: 1. }, view, ..Default::default() }
    };
    Cube.el()
        .remove(translation())
        .remove(scale())
        .remove(rotation())
        .with(local_to_world(), demo_cam.projection_view().inverse())
        .with(mesh_to_local(), Mat4::from_scale_rotation_translation(vec3(1., 1., 0.5), Quat::IDENTITY, vec3(0., 0., 0.5)))
        .with(color(), vec4(1., 0., 0., 0.5))
        .with_default(cast_shadows())
        .spawn_static(world);

    for i in 0..5 {
        let conf = RendererConfig::default();
        let shadow_cam = demo_cam.create_snapping_shadow_camera(sun_direction, i, 5, conf.shadow_map_resolution);
        Cube.el()
            .remove(translation())
            .remove(scale())
            .remove(rotation())
            .with(local_to_world(), shadow_cam.projection_view().inverse())
            .with(mesh_to_local(), Mat4::from_scale_rotation_translation(vec3(1., 1., 0.5), Quat::IDENTITY, vec3(0., 0., 0.5)))
            .with(color(), vec4(0.7, 0.7, 0.7, 1.))
            .with_default(cast_shadows())
            .spawn_static(world);

        for point in &demo_cam.to_shadows_far_bound().world_space_frustum_points_for_shadow_cascade(i, 5) {
            Cube.el().with(translation(), *point).with(color(), vec4(0., 0., 0., 1.)).with_default(cast_shadows()).spawn_static(world);
        }
    }

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .with(active_camera(), 0.)
        .with(main_scene(), ())
        .with(near(), 1.)
        .with(far(), 8000.)
        .spawn(world);

    app.world.add_component(app.world.resource_entity(), gizmo_state(), GizmoState::default()).unwrap();
    app.systems.add(Box::new(FnSystem::new(|world, _| world.resource(gizmo_state()).update(world))));

    app.window_event_systems.add(Box::new(FnSystem::new(|world, event| {
        if let Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } = event {
            if let Some(keycode) = input.virtual_keycode {
                if input.state == ElementState::Pressed {
                    world.resource_mut(gizmo_state()).on_key(keycode);
                }
            }
        }
    })));
    // cameras::free::new(
    //     vec3(-1., 0., 0.),
    //     vec2(0., 0.),
    // )   .set(active_camera(), 0.)
    //     .set(main_scene(), ())
    //     .set(near(), 1.)
    //     .set(far(), 8000.)
    //     .spawn(world);
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GizmoState {
    shadow_cameras: bool,
    bounds: bool,
}

impl GizmoState {
    pub fn update(&self, world: &World) {
        let mut scope = world.resource(gizmos()).scope(line_hash!());
        if self.shadow_cameras {
            unimplemented!()
            // scope.draw(ShadowCamera::draw_gizmos(world));
        }
        if self.bounds {
            let _gizmos = world.resource(gizmos());
            scope.draw(
                query((world_bounding_sphere(),))
                    .iter(world, None)
                    .map(|(_, (bounding,))| GizmoPrimitive::torus(bounding.center, bounding.radius, 0.1)),
            );
        }
    }

    pub fn on_key(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::Key1 => {
                self.shadow_cameras = !self.shadow_cameras;
            }
            VirtualKeyCode::Key2 => {
                self.bounds = !self.bounds;
            }
            _ => {}
        }
    }
}

ambient_ecs::components!("renderer", {
    @[Resource]
    gizmo_state: GizmoState,
});

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    init_components();
    AppBuilder::simple().block_on(init);
}
