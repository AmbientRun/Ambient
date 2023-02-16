use crate::helpers::{get_shapes, scale_shape};
use glam::{Quat, Vec3};
use kiwi_core::transform::{rotation, scale, translation};
use kiwi_ecs::{
    components, ensure_has_component, query, Debuggable, Description, FnSystem, Name, Networked, QueryState, Resource, Store, SystemGroup,
};
use kiwi_std::asset_cache::SyncAssetKey;
use parking_lot::Mutex;
use physxx::{articulation_reduced_coordinate::*, *};
use std::sync::Arc;

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
    @[Debuggable, Networked, Store, Name["Physics controlled"], Description["If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider."]]
    physics_controlled: (),
    @[Debuggable, Networked, Store, Name["Linear velocity"], Description["Linear velocity (meters/second) of this entity in the physics scene.\nUpdating this component will update the entity's linear velocity in the physics scene."]]
    linear_velocity: Vec3,
    @[Debuggable, Networked, Store, Name["Angular velocity"], Description["Angular velocity (radians/second) of this entity in the physics scene.\nUpdating this component will update the entity's angular velocity in the physics scene."]]
    angular_velocity: Vec3,
});

#[derive(Debug)]
pub struct PhysicsKey;
impl SyncAssetKey<Physics> for PhysicsKey {
    fn load(&self, _assets: kiwi_std::asset_cache::AssetCache) -> Physics {
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
    let translation_rotation_q = query((translation().changed(), rotation().changed())).incl(physics_controlled());
    let translation_rotation_q2 = translation_rotation_q.query.clone();

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
            query(physics_shape()).excl(rigid_dynamic()).excl(rigid_static()).to_system(|q, world, qs, _| {
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
                        if let Ok(body) = world.get(id, rigid_dynamic()) {
                            body.set_global_pose(&PxTransform::new(pos, rot), true);
                            body.set_linear_velocity(Vec3::ZERO, true);
                            body.set_angular_velocity(Vec3::ZERO, true);
                        } else if let Ok(body) = world.get(id, rigid_static()) {
                            body.set_global_pose(&PxTransform::new(pos, rot), true);
                        } else if let Ok(shape) = world.get_ref(id, physics_shape()) {
                            let actor = shape.get_actor().unwrap();

                            actor.set_global_pose(&PxTransform::new(pos, rot), true);
                            if let Some(body) = actor.to_rigid_dynamic() {
                                // Stop any rb movement when translating
                                body.set_linear_velocity(Vec3::ZERO, true);
                                body.set_angular_velocity(Vec3::ZERO, true);
                            } else {
                                // update_actor_entity_transforms(world, actor);
                            }
                        } else if let Ok(controller) = world.get(id, character_controller()) {
                            controller.set_position(pos.as_dvec3());
                        }
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
            // Sync PhysX changes to ECS.
            query((rigid_dynamic(), translation(), rotation())).incl(physics_controlled()).to_system(|q, world, qs, _| {
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
            query((rigid_actor(), translation(), rotation())).incl(physics_controlled()).to_system(|q, world, qs, _| {
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
            query((physics_shape(), translation(), rotation())).incl(physics_controlled()).to_system(move |q, world, qs, _| {
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
            query((articulation_link(), translation(), rotation())).incl(physics_controlled()).to_system(|q, world, qs, _| {
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
            query((character_controller(), translation())).incl(physics_controlled()).to_system(|q, world, qs, _| {
                for (id, (character_controller, pos)) in q.collect_cloned(world, qs) {
                    let new_pos = character_controller.get_foot_position().as_vec3();
                    if vec3_changed(pos, new_pos) {
                        world.set(id, translation(), new_pos).unwrap();
                    }
                }
            }),
            query(linear_velocity()).incl(physics_controlled()).to_system(|q, world, qs, _| {
                for (id, vel) in q.collect_cloned(world, qs) {
                    if let Ok(body) = world.get(id, rigid_dynamic()) {
                        let new_vel = body.get_linear_velocity();
                        if vec3_changed(vel, new_vel) {
                            world.set(id, linear_velocity(), new_vel).unwrap();
                        }
                    }
                }
            }),
            query(angular_velocity()).incl(physics_controlled()).to_system(|q, world, qs, _| {
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
