use std::sync::Arc;

use elements_core::transform::{rotation, translation};
use elements_ecs::{components, query, Networked, Store, SystemGroup};
use elements_std::asset_cache::SyncAssetKey;
use glam::{Quat, Vec3};
use physxx::{articulation_reduced_coordinate::*, *};

components!("physics", {
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
    @[Networked, Store]
    physics_controlled: (),
});

#[derive(Debug)]
pub struct PhysicsKey;
impl SyncAssetKey<Arc<Physics>> for PhysicsKey {
    fn load(&self, _assets: elements_std::asset_cache::AssetCache) -> Arc<Physics> {
        Arc::new(Physics::new())
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
        let mut cooking = PxCookingParams::new(&physics);
        cooking.0.meshWeldTolerance = 0.2;
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

fn pos_changed(old: Vec3, new: Vec3) -> bool {
    (new - old).length() > 0.001
}
fn rot_changed(old: Quat, new: Quat) -> bool {
    !old.abs_diff_eq(new, 0.001)
}

/// Syncs physx to the ECS
pub fn sync_ecs_physics() -> SystemGroup {
    let mut new_positions = Vec::new();
    let mut new_rotations = Vec::new();

    SystemGroup::new(
        "sync_ecs_physics",
        vec![
            query(()).incl(physics_controlled()).excl(translation()).to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    world.add_component(id, translation(), Vec3::ZERO).unwrap();
                }
            }),
            query(()).incl(physics_controlled()).excl(rotation()).to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    world.add_component(id, rotation(), Quat::IDENTITY).unwrap();
                }
            }),
            // Updates ecs position from physx
            query((rigid_dynamic(), translation(), rotation())).incl(physics_controlled()).to_system(|q, world, qs, _| {
                for (id, (rigid_dynamic, pos, rot)) in q.collect_cloned(world, qs) {
                    let pose = rigid_dynamic.get_global_pose();
                    let new_pos = pose.translation();
                    let new_rot = pose.rotation();
                    if pos_changed(pos, new_pos) {
                        world.set(id, translation(), new_pos).unwrap();
                    }
                    if rot_changed(rot, new_rot) {
                        world.set(id, rotation(), new_rot).unwrap();
                    }
                }
            }),
            query((rigid_actor(), translation(), rotation())).incl(physics_controlled()).to_system(|q, world, qs, _| {
                for (id, (rigid_actor, pos, rot)) in q.collect_cloned(world, qs) {
                    let pose = rigid_actor.get_global_pose();
                    let new_pos = pose.translation();
                    let new_rot = pose.rotation();
                    if pos_changed(pos, new_pos) {
                        world.set(id, translation(), new_pos).unwrap();
                    }
                    if rot_changed(rot, new_rot) {
                        world.set(id, rotation(), new_rot).unwrap();
                    }
                }
            }),
            query((physics_shape(), translation(), rotation())).incl(physics_controlled()).to_system(move |q, world, qs, _| {
                for (id, (shape, pos, rot)) in q.iter(world, qs) {
                    let actor = shape.get_actor().unwrap();
                    let global_pose = actor.get_global_pose().to_mat4();

                    let (_, new_rot, new_pos) = (global_pose).to_scale_rotation_translation();
                    if pos_changed(*pos, new_pos) {
                        new_positions.push((id, new_pos));
                    }
                    if rot_changed(*rot, new_rot) {
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
                    if pos_changed(pos, new_pos) {
                        world.set(id, translation(), new_pos).unwrap();
                    }
                    if rot_changed(rot, new_rot) {
                        world.set(id, rotation(), new_rot).unwrap();
                    }
                }
            }),
            query((character_controller(), translation())).incl(physics_controlled()).to_system(|q, world, qs, _| {
                for (id, (character_controller, pos)) in q.collect_cloned(world, qs) {
                    let new_pos = character_controller.get_foot_position().as_vec3();
                    if pos_changed(pos, new_pos) {
                        world.set(id, translation(), new_pos).unwrap();
                    }
                }
            }),
        ],
    )
}
