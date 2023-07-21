use crate::{
    collider::{collider_shapes, collider_shapes_convex},
    main_physics_scene,
    physx::{physics_shape, rigid_actor},
    picking_scene, trigger_areas_scene,
};
use ambient_core::{
    camera::Camera,
    main_scene,
    transform::{get_world_transform, local_to_world},
    ui_scene,
};
use ambient_ecs::{
    components, dont_store, ensure_has_component, ensure_has_component_with_default,
    generated::components::core::rect::{background_color, line_from, line_to, line_width},
    query, Entity, EntityId, FnSystem, SystemGroup, World,
};
use ambient_gizmos::{gizmos, GizmoPrimitive};
use ambient_std::line_hash;

use glam::{vec4, Vec3};
use itertools::Itertools;
use physxx::{
    PxActor, PxDebugLine, PxRenderBuffer, PxRigidActor, PxSceneRef, PxShape, PxShapeFlag,
    PxVisualizationParameter,
};

pub use ambient_ecs::generated::components::core::physics::visualize_collider;

components!("physics", {
    physx_viz_line: PxDebugLine,
    shape_primitives: Vec<GizmoPrimitive>,
});

pub fn run_visualize_collider(world: &mut World, entity: EntityId, enabled: bool) -> Option<()> {
    let actor = world.get_ref(entity, rigid_actor()).ok()?;
    let scene = actor.get_scene()?;
    for shape in world
        .get_ref(entity, collider_shapes())
        .into_iter()
        .flatten()
    {
        visualize_shape(scene, shape, enabled);
    }

    for shape in world
        .get_ref(entity, collider_shapes_convex())
        .into_iter()
        .flatten()
    {
        visualize_shape(scene, shape, enabled)
    }

    if enabled {
        world
            .add_component(entity, visualize_collider(), ())
            .unwrap();
    } else {
        world.remove_component(entity, visualize_collider()).ok();
    }
    Some(())
}

fn visualize_shape(scene: PxSceneRef, shape: &PxShape, enabled: bool) {
    shape.set_flag(PxShapeFlag::VISUALIZATION, enabled);

    scene.set_visualization_parameter(PxVisualizationParameter::SCALE, 10.0);
    scene.set_visualization_parameter(PxVisualizationParameter::JOINT_LOCAL_FRAMES, 1.0);
    scene.set_visualization_parameter(PxVisualizationParameter::JOINT_LIMITS, 1.0);
    // scene.set_visualization_parameter(PxVisualizationParameter::ACTOR_AXES, 1.0);
    scene.set_visualization_parameter(PxVisualizationParameter::COLLISION_SHAPES, 1.0);
}

pub fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "visualization/server",
        vec![
            // This is needed as duplicating an object does not carry over the physx flags, but
            // the object is still recognized as visualize_collider
            query((visualize_collider(), collider_shapes().changed())).to_system_with_name(
                "visualization/ensure_visualization",
                |q, w, qs, _| {
                    for id in q.collect_ids(w, qs) {
                        run_visualize_collider(w, id, true);
                    }
                },
            ),
            query((visualize_collider(), collider_shapes()))
                .spawned()
                .to_system_with_name(
                    "visualization/ensure_visualization_spawned",
                    |q, w, qs, _| {
                        for id in q.collect_ids(w, qs) {
                            run_visualize_collider(w, id, true);
                        }
                    },
                ),
            query(visualize_collider()).despawned().to_system_with_name(
                "visualization/despawned",
                |q, w, qs, _| {
                    ambient_profiling::scope!("server_shape_visualize_remove");
                    let mut ids = Vec::new();
                    for (id, _) in q.iter(w, qs) {
                        ids.push(id);
                    }

                    for id in ids {
                        tracing::info!("Removing visualize_collider from {id:?}");
                        w.remove_component(id, shape_primitives()).ok();
                    }
                },
            ),
            query(visualize_collider()).to_system_with_name(
                "visualization/shapes",
                |q, w, qs, _| {
                    ambient_profiling::scope!("server_shape_visualize");
                    let mut primitives = Vec::new();

                    for (id, ()) in q.iter(w, qs) {
                        let ltw = get_world_transform(w, id).unwrap_or_default();

                        let mut current = Vec::new();

                        let (_, _, pos) = ltw.to_scale_rotation_translation();

                        current.push(GizmoPrimitive::sphere(pos, 0.15).with_color(Vec3::X));

                        if let Ok(shape) = w.get_ref(id, physics_shape()) {
                            let actor = shape.get_actor().unwrap();
                            current.push(
                                GizmoPrimitive::sphere(actor.get_global_pose().translation(), 0.1)
                                    .with_color(Vec3::Y),
                            );
                        }

                        primitives.push((id, current))
                    }

                    for (id, p) in primitives {
                        w.add_component(id, shape_primitives(), p)
                            .expect("Invalid component");
                    }
                },
            ),
            Box::new(FnSystem::new(|world, _| {
                let mut render_buffer = PxRenderBuffer::default();
                for scene in [main_physics_scene(), picking_scene(), trigger_areas_scene()] {
                    let scene = world.resource(scene);
                    if scene.get_visualization_parameter(PxVisualizationParameter::SCALE) > 0. {
                        let rb = scene.get_render_buffer();
                        render_buffer.points.extend(rb.points.into_iter());
                        render_buffer.lines.extend(rb.lines.into_iter());
                    }
                }

                let existing = query(())
                    .incl(physx_viz_line())
                    .iter(world, None)
                    .map(|(id, _)| id)
                    .collect_vec();
                for (entity, line) in existing.iter().zip(render_buffer.lines.iter()) {
                    let _ = world.set_if_changed(*entity, physx_viz_line(), line.clone());
                }

                #[allow(clippy::comparison_chain)]
                if render_buffer.lines.len() > existing.len() {
                    for i in existing.len()..render_buffer.lines.len() {
                        Entity::new()
                            .with_default(dont_store())
                            .with(physx_viz_line(), render_buffer.lines[i].clone())
                            .spawn(world);
                    }
                } else if existing.len() > render_buffer.lines.len() {
                    for entity in existing.iter().skip(render_buffer.lines.len()) {
                        world.despawn(*entity);
                    }
                }
            })),
        ],
    )
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "visualization/client",
        vec![
            ensure_has_component_with_default(physx_viz_line(), line_from()),
            ensure_has_component_with_default(physx_viz_line(), line_to()),
            ensure_has_component(physx_viz_line(), line_width(), 1.),
            ensure_has_component(physx_viz_line(), background_color(), vec4(1., 0., 0., 1.)),
            ensure_has_component_with_default(physx_viz_line(), ui_scene()),
            ensure_has_component_with_default(physx_viz_line(), local_to_world()),
            query((physx_viz_line(),)).to_system(|q, world, qs, _| {
                if let Some(world_cam) = Camera::get_active(world, main_scene(), None) {
                    if let Some(ui_cam) = Camera::get_active(world, ui_scene(), None) {
                        let mat = ui_cam.projection_view().inverse() * world_cam.projection_view();

                        for (id, (line,)) in q.collect_cloned(world, qs) {
                            let from = mat.project_point3(line.pos0);
                            let to = mat.project_point3(line.pos1);
                            world.set_if_changed(id, line_from(), from).unwrap();
                            world.set_if_changed(id, line_to(), to).unwrap();
                        }
                    }
                }
            }),
            query((shape_primitives(),)).to_system(|q, world, qs, _| {
                ambient_profiling::scope!("shape_gizmo_render");
                let mut scope = world.resource(gizmos()).scope(line_hash!());
                for (_, (prim,)) in q.iter(world, qs) {
                    scope.draw(prim.iter().copied());
                }
            }),
        ],
    )
}
