use std::sync::Arc;

use ambient_core::{
    dtime,
    transform::{rotation, scale, translation},
};
use ambient_ecs::{
    components, ensure_has_component, query, FnSystem, QueryState, Resource, SystemGroup,
};
use ambient_std::asset_cache::SyncAssetKey;
use glam::{EulerRot, Quat, Vec3};
use parking_lot::Mutex;
use physxx::{articulation_reduced_coordinate::*, *};

use crate::helpers::{get_shapes, scale_shape};

pub use ambient_ecs::generated::components::core::physics::*;

components!("physics", {
    @[Resource]
    physics: Physics,
    actor_aggregate: PxAggregateRef,
    rigid_actor: PxRigidActorRef,
    rigid_dynamic: PxRigidDynamicRef,
    rigid_static: PxRigidStaticRef,
    physics_shape: PxShape,
    fixed_joint: PxFixedJointRef,
    revolute_joint: PxRevoluteJointRef,
    articulation_reduce_coordinate: PxArticulationRef,
    articulation_link: PxArticulationLinkRef,
    articulation_cache: Option<PxArticulationCacheRef>,
    character_controller: PxControllerRef,
});

#[derive(Debug)]
pub struct PhysicsKey;
impl SyncAssetKey<Physics> for PhysicsKey {
    fn load(&self, _assets: ambient_std::asset_cache::AssetCache) -> Physics {
        Physics::new()
    }
}

#[derive(Clone)]
pub struct Physics {
    pub foundation: PxFoundationRef,
    pub physics: PxPhysicsRef,
    pub dispatcher: PxDefaultCpuDispatcherRef,
    pub cooking: PxCookingRef,
    pub pvd: PxPvdRef,
    pub pvd_transport: PxPvdTransportRef,
    pub serialization_registry: PxSerializationRegistryRef,
}
impl Physics {
    pub fn new() -> Self {
        let foundation = PxFoundationRef::new();
        let pvd = PxPvdRef::new(&foundation);
        let pvd_transport = PxPvdTransportRef::new();
        pvd.connect(&pvd_transport, PxPvdInstrumentationFlags::All);
        let physics = PxPhysicsRef::new_with_pvd(&foundation, &pvd);
        px_init_extensions(&physics, &pvd);
        let mut cooking = PxCookingParams::new(physics);
        cooking.0.meshWeldTolerance = 0.001; // 1mm precision
        cooking.0.meshPreprocessParams.mBits = physxx::sys::PxMeshPreprocessingFlag::eWELD_VERTICES;
        Self {
            serialization_registry: PxSerializationRegistryRef::new(&physics),
            cooking: PxCookingRef::new(&foundation, &cooking),
            foundation,
            pvd,
            pvd_transport,
            physics,
            dispatcher: PxDefaultCpuDispatcherRef::new(2),
        }
    }
    pub fn release(self) {
        self.serialization_registry.release();
        self.cooking.release();
        self.pvd.release();
        self.pvd_transport.release();
        self.dispatcher.release();
        self.physics.release();
        self.foundation.release();
    }
}

fn vec3_changed(old: Vec3, new: Vec3) -> bool {
    (new - old).length() > 0.001
}
fn quat_changed(old: Quat, new: Quat) -> bool {
    !old.abs_diff_eq(new, 0.001)
}

/// Syncs physx to the ECS
pub fn sync_ecs_physics() -> SystemGroup {
    let mut new_positions = Vec::new();
    let mut new_rotations = Vec::new();

    let translation_rotation_qs = Arc::new(Mutex::new(QueryState::new()));
    let translation_rotation_q =
        query((translation().changed(), rotation().changed())).incl(physics_shape());
    let translation_rotation_q2 = translation_rotation_q.query.clone();

    let translation_character_qs = Arc::new(Mutex::new(QueryState::new()));
    let translation_character_q =
        query((translation().changed(), character_controller().changed()))
            .incl(physics_controlled());
    let translation_character_q2 = translation_character_q.query.clone();

    let scale_qs = Arc::new(Mutex::new(QueryState::new()));
    let scale_q = query(scale().changed()).incl(physics_controlled());
    let scale_q2 = scale_q.query.clone();

    let linear_velocity_qs = Arc::new(Mutex::new(QueryState::new()));
    let linear_velocity_q = query(linear_velocity().changed()).incl(physics_controlled());
    let linear_velocity_q2 = scale_q.query.clone();

    let angular_velocity_qs = Arc::new(Mutex::new(QueryState::new()));
    let angular_velocity_q = query(angular_velocity().changed()).incl(physics_controlled());
    let angular_velocity_q2 = scale_q.query.clone();

    SystemGroup::new(
        "sync_ecs_physics",
        vec![
            // Ensure that physics-related components are added if required.
            ensure_has_component(physics_controlled(), translation(), Vec3::ZERO),
            ensure_has_component(physics_controlled(), rotation(), Quat::IDENTITY),
            query(physics_shape())
                .excl(rigid_dynamic())
                .excl(rigid_static())
                .to_system(|q, world, qs, _| {
                    for (id, shape) in q.collect_cloned(world, qs) {
                        let Some(actor) = shape.get_actor() else { continue; };

                        if let Some(body) = actor.to_rigid_dynamic() {
                            world.add_component(id, rigid_dynamic(), body).unwrap();
                        } else if let Some(body) = actor.to_rigid_static() {
                            world.add_component(id, rigid_static(), body).unwrap();
                        }
                    }
                }),
            ensure_has_component(rigid_dynamic(), linear_velocity(), Vec3::default()),
            ensure_has_component(rigid_dynamic(), angular_velocity(), Vec3::default()),
            // Sync ECS changes to PhysX.
            translation_rotation_q.to_system({
                let translation_rotation_qs = translation_rotation_qs.clone();
                move |q, world, _, _| {
                    // Read back any changes that have happened during the frame to physx
                    let mut qs = translation_rotation_qs.lock();
                    for (id, (&pos, &rot)) in q.iter(world, Some(&mut *qs)) {
                        let is_kinematic = world.has_component(id, kinematic());
                        if let Ok(body) = world.get(id, rigid_dynamic()) {
                            let pose = PxTransform::new(pos, rot);
                            if is_kinematic {
                                body.set_kinematic_target(&pose);
                            } else {
                                body.set_global_pose(&pose, true);
                                body.set_linear_velocity(Vec3::ZERO, true);
                                body.set_angular_velocity(Vec3::ZERO, true);
                            }
                        } else if let Ok(body) = world.get(id, rigid_static()) {
                            body.set_global_pose(&PxTransform::new(pos, rot), true);
                        } else if let Ok(shape) = world.get_ref(id, physics_shape()) {
                            let actor = shape.get_actor().unwrap();
                            let pose = PxTransform::new(pos, rot);
                            if !is_kinematic {
                                actor.set_global_pose(&pose, true);
                            }
                            if let Some(body) = actor.to_rigid_dynamic() {
                                if is_kinematic {
                                    body.set_kinematic_target(&pose);
                                } else {
                                    // Stop any rb movement when translating
                                    body.set_linear_velocity(Vec3::ZERO, true);
                                    body.set_angular_velocity(Vec3::ZERO, true);
                                }
                            } else {
                                // update_actor_entity_transforms(world, actor);
                            }
                        }
                    }
                }
            }),
            translation_character_q.to_system({
                let translation_character_qs = translation_character_qs.clone();
                move |q, world, _, _| {
                    let mut qs = translation_character_qs.lock();
                    for (_, (&pos, controller)) in q.iter(world, Some(&mut *qs)) {
                        controller.set_position(pos.as_dvec3());
                    }
                }
            }),
            scale_q.to_system({
                let scale_qs = scale_qs.clone();
                move |q, world, _, _| {
                    let mut qs = scale_qs.lock();
                    for (id, &scl) in q.iter(world, Some(&mut *qs)) {
                        for shape in get_shapes(world, id) {
                            scale_shape(shape, scl);
                        }
                    }
                }
            }),
            linear_velocity_q.to_system({
                let linear_velocity_qs = linear_velocity_qs.clone();
                move |q, world, _, _| {
                    let mut qs = linear_velocity_qs.lock();
                    for (id, &vel) in q.iter(world, Some(&mut *qs)) {
                        if let Ok(body) = world.get(id, rigid_dynamic()) {
                            body.set_linear_velocity(vel, true);
                        }
                    }
                }
            }),
            angular_velocity_q.to_system({
                let angular_velocity_qs = angular_velocity_qs.clone();
                move |q, world, _, _| {
                    let mut qs = angular_velocity_qs.lock();
                    for (id, &vel) in q.iter(world, Some(&mut *qs)) {
                        if let Ok(body) = world.get(id, rigid_dynamic()) {
                            body.set_angular_velocity(vel, true);
                        }
                    }
                }
            }),
            query((
                rigid_dynamic(),
                translation(),
                rotation(),
                linear_velocity(),
                angular_velocity(),
            ))
            .incl(kinematic())
            .to_system(|q, world, qs, _| {
                let dtime = *world.resource(dtime());
                for (id, (body, pos, rot, lvel, avel)) in q.collect_cloned(world, qs) {
                    let avel = avel * dtime;
                    let new_pos = pos + lvel * dtime;
                    let new_rot = rot * Quat::from_euler(EulerRot::XYZ, avel.x, avel.y, avel.z);
                    let pos_changed = vec3_changed(pos, new_pos);
                    let rot_changed = quat_changed(rot, new_rot);
                    if pos_changed {
                        world.set(id, translation(), new_pos).unwrap();
                    }
                    if rot_changed {
                        world.set(id, rotation(), new_rot).unwrap();
                    }
                    if pos_changed || rot_changed {
                        body.set_kinematic_target(&PxTransform::new(new_pos, new_rot));
                    }
                }
            }),
            query(contact_offset().changed())
                .incl(physics_controlled())
                .to_system(|q, world, qs, _| {
                    for (id, &off) in q.iter(world, qs) {
                        for shape in get_shapes(world, id) {
                            shape.set_contact_offset(off);
                        }
                    }
                }),
            query(rest_offset().changed())
                .incl(physics_controlled())
                .to_system(|q, world, qs, _| {
                    for (id, &off) in q.iter(world, qs) {
                        for shape in get_shapes(world, id) {
                            shape.set_rest_offset(off);
                        }
                    }
                }),
            // Sync PhysX changes to ECS.
            query((rigid_dynamic(), translation(), rotation()))
                .incl(physics_controlled())
                .to_system(|q, world, qs, _| {
                    for (id, (rigid_dynamic, pos, rot)) in q.collect_cloned(world, qs) {
                        let pose = rigid_dynamic.get_global_pose();
                        let new_pos = pose.translation();
                        let new_rot = pose.rotation();
                        if vec3_changed(pos, new_pos) {
                            world.set(id, translation(), new_pos).unwrap();
                        }
                        if quat_changed(rot, new_rot) {
                            world.set(id, rotation(), new_rot).unwrap();
                        }
                    }
                }),
            query((rigid_actor(), translation(), rotation()))
                .incl(physics_controlled())
                .to_system(|q, world, qs, _| {
                    for (id, (rigid_actor, pos, rot)) in q.collect_cloned(world, qs) {
                        let pose = rigid_actor.get_global_pose();
                        let new_pos = pose.translation();
                        let new_rot = pose.rotation();
                        if vec3_changed(pos, new_pos) {
                            world.set(id, translation(), new_pos).unwrap();
                        }
                        if quat_changed(rot, new_rot) {
                            world.set(id, rotation(), new_rot).unwrap();
                        }
                    }
                }),
            query((physics_shape(), translation(), rotation()))
                .incl(physics_controlled())
                .to_system(move |q, world, qs, _| {
                    for (id, (shape, pos, rot)) in q.iter(world, qs) {
                        let actor = shape.get_actor().unwrap();
                        let global_pose = actor.get_global_pose().to_mat4();

                        let (_, new_rot, new_pos) = (global_pose).to_scale_rotation_translation();
                        if vec3_changed(*pos, new_pos) {
                            new_positions.push((id, new_pos));
                        }
                        if quat_changed(*rot, new_rot) {
                            new_rotations.push((id, new_rot));
                        }
                    }

                    for (id, pos) in new_positions.drain(..) {
                        world.set(id, translation(), pos).unwrap();
                    }
                    for (id, rot) in new_rotations.drain(..) {
                        world.set(id, rotation(), rot).unwrap();
                    }
                }),
            query((articulation_link(), translation(), rotation()))
                .incl(physics_controlled())
                .to_system(|q, world, qs, _| {
                    for (id, (articulation_link, pos, rot)) in q.collect_cloned(world, qs) {
                        let pose = articulation_link.get_global_pose();
                        let new_pos = pose.translation();
                        let new_rot = pose.rotation();
                        if vec3_changed(pos, new_pos) {
                            world.set(id, translation(), new_pos).unwrap();
                        }
                        if quat_changed(rot, new_rot) {
                            world.set(id, rotation(), new_rot).unwrap();
                        }
                    }
                }),
            query((character_controller(), translation()))
                .incl(physics_controlled())
                .to_system(|q, world, qs, _| {
                    for (id, (character_controller, pos)) in q.collect_cloned(world, qs) {
                        let new_pos = character_controller.get_foot_position().as_vec3();
                        if vec3_changed(pos, new_pos) {
                            world.set(id, translation(), new_pos).unwrap();
                        }
                    }
                }),
            query(linear_velocity())
                .incl(physics_controlled())
                .to_system(|q, world, qs, _| {
                    for (id, vel) in q.collect_cloned(world, qs) {
                        if let Ok(body) = world.get(id, rigid_dynamic()) {
                            let new_vel = body.get_linear_velocity();
                            if vec3_changed(vel, new_vel) {
                                world.set(id, linear_velocity(), new_vel).unwrap();
                            }
                        }
                    }
                }),
            query(angular_velocity())
                .incl(physics_controlled())
                .to_system(|q, world, qs, _| {
                    for (id, vel) in q.collect_cloned(world, qs) {
                        if let Ok(body) = world.get(id, rigid_dynamic()) {
                            let new_vel = body.get_angular_velocity();
                            if vec3_changed(vel, new_vel) {
                                world.set(id, angular_velocity(), new_vel).unwrap();
                            }
                        }
                    }
                }),
            Box::new(FnSystem::new(move |world, _| {
                // Fast forward queries
                let mut translation_rotation_qs = translation_rotation_qs.lock();
                for _ in translation_rotation_q2.iter(world, Some(&mut *translation_rotation_qs)) {}
                let mut translation_character_qs = translation_character_qs.lock();
                for _ in translation_character_q2.iter(world, Some(&mut *translation_character_qs))
                {
                }
                let mut scale_qs = scale_qs.lock();
                for _ in scale_q2.iter(world, Some(&mut *scale_qs)) {}
                let mut linear_velocity_qs = linear_velocity_qs.lock();
                for _ in linear_velocity_q2.iter(world, Some(&mut *linear_velocity_qs)) {}
                let mut angular_velocity_qs = angular_velocity_qs.lock();
                for _ in angular_velocity_q2.iter(world, Some(&mut *angular_velocity_qs)) {}
            })),
        ],
    )
}
