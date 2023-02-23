use std::collections::HashSet;

use ambient_core::transform::{get_world_position, rotation, translation};
use ambient_ecs::{query, ECSError, EntityId, World};
use anyhow::{bail, Context};
use glam::{vec3, Vec3};
use itertools::Itertools;
use physxx::{
    AsPxActor, AsPxRigidActor, PxActor, PxActorTypeFlag, PxBase, PxBoxGeometry, PxConvexMeshGeometry, PxJoint, PxMeshScale, PxOverlapCallback, PxQueryFilterData, PxQueryFlag, PxRevoluteJointRef, PxRigidActor, PxRigidActorRef, PxRigidBody, PxRigidBodyFlag, PxRigidDynamicRef, PxRigidStaticRef, PxSceneRef, PxShape, PxSphereGeometry, PxTransform, PxTriangleMeshGeometry, PxUserData
};

use crate::{
    collider::{collider_shapes_convex, collider_type, kinematic}, main_physics_scene, physx::{physics, physics_controlled, physics_shape, revolute_joint, rigid_dynamic}, unit_mass, unit_velocity, ColliderScene, PxActorUserData, PxShapeUserData
};

pub fn convert_rigid_static_to_dynamic(world: &mut World, id: EntityId) {
    convert_rigid_static_dynamic(world, id, true);
}
pub fn convert_rigid_dynamic_to_static(world: &mut World, id: EntityId) {
    convert_rigid_static_dynamic(world, id, false);
}

pub fn convert_rigid_static_dynamic(world: &mut World, id: EntityId, to_dynamic: bool) {
    let old_actor = {
        if let Ok(shape) = world.get_ref(id, physics_shape()) {
            shape.get_actor().unwrap()
        } else {
            return;
        }
    };
    if (to_dynamic && old_actor.to_rigid_dynamic().is_some()) || (!to_dynamic && old_actor.to_rigid_static().is_some()) {
        return;
    }
    let shapes = old_actor.get_shapes();
    let constraints = old_actor.get_constraints();
    let scene = old_actor.get_scene().unwrap();
    scene.remove_actor(&old_actor, true);
    let physics = world.resource(physics());
    let new_actor = if to_dynamic {
        let actor = PxRigidDynamicRef::new(physics.physics, &old_actor.get_global_pose());
        let is_kinematic = world.has_component(id, kinematic());
        actor.set_rigid_body_flag(PxRigidBodyFlag::KINEMATIC, is_kinematic);
        actor.set_rigid_body_flag(PxRigidBodyFlag::ENABLE_CCD, !is_kinematic);
        actor.as_rigid_actor()
    } else {
        PxRigidStaticRef::new(physics.physics, &old_actor.get_global_pose()).as_rigid_actor()
    };
    new_actor.as_actor().set_user_data(old_actor.as_actor().get_user_data::<PxActorUserData>().unwrap());
    for shape in shapes {
        old_actor.detach_shape(&shape, false);
        new_actor.attach_shape(&shape);
    }
    for constraint in constraints {
        let joint = constraint.get_external_reference().to_joint().unwrap();
        let (a0, a1) = joint.get_actors();
        if a0 == Some(old_actor) {
            joint.set_actors(Some(new_actor), a1);
        } else {
            joint.set_actors(a0, Some(new_actor));
        }
    }
    old_actor.as_actor().remove_user_data::<PxActorUserData>();
    old_actor.release();
    if let Some(actor) = new_actor.to_rigid_dynamic() {
        let densities = actor.get_shapes().iter().map(|shape| shape.get_user_data::<PxShapeUserData>().unwrap().density).collect_vec();
        actor.update_mass_and_inertia(densities, None, None);
    }
    update_physics_controlled(world, new_actor.as_rigid_actor());
    scene.add_actor(&new_actor);
}

pub fn update_physics_controlled(world: &mut World, actor: PxRigidActorRef) {
    let is_physics_controlled = match actor.to_rigid_dynamic() {
        Some(body) => !body.get_rigid_body_flags().contains(PxRigidBodyFlag::KINEMATIC),
        _ => false,
    };
    for shape in actor.get_shapes() {
        let entity = shape.get_user_data::<PxShapeUserData>().unwrap().entity;
        if let Ok(entity_shape) = world.get_ref(entity, physics_shape()) {
            if entity_shape == &shape {
                if is_physics_controlled {
                    if !world.has_component(entity, physics_controlled()) {
                        world.add_component(entity, physics_controlled(), ()).ok();
                    }
                } else if world.has_component(entity, physics_controlled()) {
                    world.remove_component(entity, physics_controlled()).ok();
                }
            }
        }
    }
}

pub fn get_linear_velocity(world: &World, id: EntityId) -> Result<Vec3, ECSError> {
    let v = if let Ok(body) = world.get_ref(id, rigid_dynamic()) {
        body.get_linear_velocity()
    } else if let Ok(shape) = world.get_ref(id, physics_shape()) {
        shape.get_actor().unwrap().to_rigid_dynamic().map(|b| b.get_linear_velocity()).unwrap_or_default()
    } else {
        Vec3::ZERO
    };
    Ok(v)
}

/// Returns both the simulated and faked convex shapes of an entity
pub fn get_shapes(world: &World, id: EntityId) -> impl Iterator<Item = PxShape> + '_ {
    world
        .get_ref(id, physics_shape())
        .into_iter()
        .map(|v| v.get_actor().unwrap())
        // Shapes attached to the actor
        .flat_map(|v| v.get_shapes().into_iter())
        // Convex shapes used for sweeping
        .chain(world.get_ref(id, collider_shapes_convex()).into_iter().flatten().cloned())
}

pub fn scale_shape(shape: PxShape, scale: Vec3) {
    tracing::debug!("Scaling shape");
    let geo = shape.get_geometry();
    let ud = shape.get_user_data::<PxShapeUserData>().unwrap();
    let (base_scale, base_rot, base_pos) = ud.base_pose.to_scale_rotation_translation();
    // The new size
    let size = (base_rot * scale) * base_scale;

    if size.is_nan() || size.length() < 0.001 {
        tracing::warn!("Scale is nan or zero");
        return;
    }

    if let Some(geo) = geo.as_convex_mesh() {
        let mesh = geo.mesh();
        let new_geo = PxConvexMeshGeometry::new(&mesh, Some(PxMeshScale::from_scale((size).abs())), Some(geo.flags()));
        if new_geo.is_valid() {
            shape.set_geometry(&new_geo);
        } else {
            tracing::error!(?scale, ?size, ?base_scale, ?base_rot, ?base_pos, "scale_shape: Invalid convex geometry");
        }
    } else if let Some(geo) = geo.as_triangle_mesh() {
        let mesh = geo.mesh();
        let new_geo = PxTriangleMeshGeometry::new(&mesh, Some(PxMeshScale::from_scale(size)), Some(geo.flags()));
        if new_geo.is_valid() {
            shape.set_geometry(&new_geo);
        } else {
            tracing::error!(?scale, ?size, ?base_scale, ?base_rot, ?base_pos, "scale_shape: Invalid triangle geometry");
        }
    } else if let Some(_geo) = geo.as_sphere() {
        let new_geo = PxSphereGeometry::new((size).x);
        shape.set_geometry(&new_geo);
    } else if let Some(_geo) = geo.as_box() {
        let new_geo = PxBoxGeometry::new(size.x, size.y, size.z);
        shape.set_geometry(&new_geo);
    } else {
        // TODO
    }

    // Update the local pose to scale the offset along with the colliders scale
    shape.set_local_pose(&PxTransform::new(scale * base_pos, base_rot));
}

#[tracing::instrument(skip(world), level = "info")]
pub fn update_actor_entity_transforms(world: &mut World, actor: PxRigidActorRef) {
    for shape in actor.get_shapes() {
        let entity = shape.get_user_data::<PxShapeUserData>().unwrap().entity;
        if let Ok(entity_shape) = world.get_ref(entity, physics_shape()) {
            if entity_shape == &shape {
                let global_pose = actor.get_global_pose().to_mat4();
                let shape_pose = shape.get_local_pose().to_mat4();
                let (_, new_rot, new_pos) = (global_pose * shape_pose).to_scale_rotation_translation();

                world.set(entity, translation(), new_pos).unwrap();
                world.set(entity, rotation(), new_rot).unwrap();
            }
        }
    }
}

pub fn get_entity_revolute_joint(world: &World, id: EntityId) -> Option<PxRevoluteJointRef> {
    if let Ok(joint) = world.get(id, revolute_joint()) {
        return Some(joint);
    } else if let Ok(shape) = world.get_ref(id, physics_shape()) {
        let constraints = shape.get_actor()?.get_constraints();
        if !constraints.is_empty() {
            return constraints[0].get_external_reference().to_revolute_joint();
        }
    }
    None
}

pub fn weld_multi(world: &mut World, selected: Vec<EntityId>) {
    let mut selected = selected
        .into_iter()
        .filter(|id| matches!(world.get(*id, collider_type()).map(|x| x.scene()), Ok(ColliderScene::Physics)))
        .collect_vec();
    if selected.len() <= 1 {
        return;
    }

    while selected.len() >= 2 {
        let left = selected[0];
        let right = selected.remove(1);
        weld_two(world, left, right);
    }
}

pub fn weld_two(world: &mut World, first: EntityId, second: EntityId) {
    let first_shape = world.get_ref(first, physics_shape()).unwrap().clone();
    let first_actor = first_shape.get_actor().unwrap();
    let second_shape = world.get_ref(second, physics_shape()).unwrap().clone();
    let second_actor = second_shape.get_actor().unwrap();
    let scene = second_actor.get_scene().unwrap();

    if first_actor == second_actor || first_actor.get_scene() != second_actor.get_scene() {
        return;
    }
    // Prevent welding joint to itself
    for constraint in first_actor.get_constraints() {
        let joint = constraint.get_external_reference().to_joint().unwrap();
        let (a0, a1) = joint.get_actors();
        if (a0 == Some(first_actor) && a1 == Some(second_actor)) || (a1 == Some(first_actor) && a0 == Some(second_actor)) {
            return;
        }
    }
    let first_inv_global = first_actor.get_global_pose().to_mat4().inverse();
    let second_global = second_actor.get_global_pose().to_mat4();
    let shapes = second_actor
        .get_shapes()
        .into_iter()
        .map(|shape| {
            let local_pose = shape.get_local_pose().to_mat4();
            second_actor.detach_shape(&shape, false);
            let new_local_pose = first_inv_global * second_global * local_pose;
            (shape, new_local_pose)
        })
        .collect_vec();
    scene.remove_actor(&second_actor, true);
    for constraint in second_actor.get_constraints() {
        let joint = constraint.get_external_reference().to_joint().unwrap();
        let (a0, a1) = joint.get_actors();
        let can_have_joint = |other: &Option<PxRigidActorRef>| {
            if let Some(other) = other {
                (first_actor.to_rigid_dynamic().is_some() || other.to_rigid_dynamic().is_some()) && &first_actor != other
            } else {
                true
            }
        };
        if a0 == Some(second_actor) {
            if can_have_joint(&a1) {
                joint.set_actors(Some(first_actor), a1);
                let p0 = joint.get_local_pose(0).to_mat4();
                let pose_matrix = first_inv_global * second_global * p0;
                let (scale, rotation, translation) = pose_matrix.to_scale_rotation_translation();
                if (scale.length() - 1.0).abs() > f32::EPSILON {
                    panic!("Pose matrix had a non-unit scale, which is unsupported by PhysX. E1 {first} | E2 {second} | Scale: {scale:?} | Rotation: {rotation:?} | Translation: {translation:?} | pose_matrix: {pose_matrix:?} | first_inv_global: {first_inv_global:?} | second_global: {second_global:?} | p0: {p0:?}");
                }
                joint.set_local_pose(0, &PxTransform::new(translation, rotation));
            } else {
                let entity = joint.get_user_data::<EntityId>().unwrap();
                if joint.to_revolute_joint().is_some() {
                    world.remove_component(entity, revolute_joint()).ok();
                } else {
                    unimplemented!()
                }
                joint.release();
            }
        } else if can_have_joint(&a0) {
            joint.set_actors(a0, Some(first_actor));
            let p0 = joint.get_local_pose(1).to_mat4();
            let pose_matrix = first_inv_global * second_global * p0;
            let (scale, rotation, translation) = pose_matrix.to_scale_rotation_translation();
            if (scale.length() - 1.0).abs() > f32::EPSILON {
                panic!("Pose matrix had a non-unit scale, which is unsupported by PhysX. E1 {first} | E2 {second} | Scale: {scale:?} | Rotation: {rotation:?} | Translation: {translation:?} | pose_matrix: {pose_matrix:?} | first_inv_global: {first_inv_global:?} | second_global: {second_global:?} | p0: {p0:?}");
            }
            joint.set_local_pose(1, &PxTransform::new(translation, rotation));
        } else {
            let entity = joint.get_user_data::<EntityId>().unwrap();
            if joint.to_revolute_joint().is_some() {
                world.remove_component(entity, revolute_joint()).ok();
            } else {
                unimplemented!()
            }
            joint.release();
        }
    }
    second_actor.as_actor().remove_user_data::<PxActorUserData>();
    second_actor.release();
    scene.fetch_results(true);
    for (shape, new_local_pose) in shapes.into_iter() {
        let (scale, rotation, translation) = new_local_pose.to_scale_rotation_translation();
        if (scale.length() - 1.0).abs() > f32::EPSILON {
            panic!("New local pose had a non-unit scale, which is unsupported by PhysX. E1 {first} | E2 {second} | Scale: {scale:?} | Rotation: {rotation:?} | Translation: {translation:?} | new_local_pose: {new_local_pose:?}");
        }
        shape.set_local_pose(&PxTransform::new(translation, rotation));
        if !first_actor.attach_shape(&shape) {
            panic!("Failed to attach shape");
        }
    }
    if let Some(first) = first_actor.to_rigid_body() {
        let densities = first.get_shapes().iter().map(|shape| shape.get_user_data::<PxShapeUserData>().unwrap().density).collect_vec();
        first.update_mass_and_inertia(densities, None, None);
    }
    update_physics_controlled(world, first_actor);
}

pub fn unweld_multi(world: &World, selected: Vec<EntityId>) {
    let mut selected = selected
        .into_iter()
        .filter(|id| matches!(world.get(*id, collider_type()).map(|x| x.scene()), Ok(ColliderScene::Physics)))
        .collect_vec();
    if selected.len() <= 1 {
        return;
    }

    while selected.len() >= 2 {
        let left = selected[0];
        let right = selected.remove(1);
        unweld_two(world, left, right);
    }
}

pub fn unweld_two(world: &World, first: EntityId, second: EntityId) {
    let first_shape = world.get_ref(first, physics_shape()).unwrap().clone();
    let first_actor = first_shape.get_actor().unwrap();
    let second_shape = world.get_ref(second, physics_shape()).unwrap().clone();
    let second_actor = second_shape.get_actor().unwrap();
    let scene = second_actor.get_scene().unwrap();
    let physics = world.resource(physics());

    if first_actor != second_actor {
        return;
    }
    let second_shapes = first_actor
        .get_shapes()
        .into_iter()
        .filter(|shape| {
            if shape.get_user_data::<PxShapeUserData>().unwrap().entity == second {
                first_actor.detach_shape(shape, true);
                true
            } else {
                false
            }
        })
        .collect_vec();
    let second_actor = if first_actor.to_rigid_dynamic().is_some() {
        PxRigidDynamicRef::new(physics.physics, &first_actor.get_global_pose()).as_rigid_actor()
    } else {
        PxRigidStaticRef::new(physics.physics, &first_actor.get_global_pose()).as_rigid_actor()
    };
    second_actor.as_actor().set_user_data(first_actor.as_actor().get_user_data::<PxActorUserData>());
    for shape in second_shapes {
        if !second_actor.attach_shape(&shape) {
            panic!("Failed to attach shape");
        }
    }
    scene.add_actor(&second_actor);

    if let Some(first) = first_actor.to_rigid_body() {
        let densities = first.get_shapes().iter().map(|shape| shape.get_user_data::<PxShapeUserData>().unwrap().density).collect_vec();
        first.update_mass_and_inertia(densities, None, None);
    }

    if let Some(second) = second_actor.to_rigid_body() {
        let densities = second.get_shapes().iter().map(|shape| shape.get_user_data::<PxShapeUserData>().unwrap().density).collect_vec();
        second.update_mass_and_inertia(densities, None, None);
    }
}

#[derive(Debug, Default)]
pub struct PhysicsObjectCollection {
    actors: Vec<PxRigidActorRef>,
    units: Vec<EntityId>,
}
impl PhysicsObjectCollection {
    pub fn from_entity(world: &World, entity: EntityId) -> Self {
        Self::from_entities(world, &[entity])
    }
    pub fn from_entities(world: &World, entities: &[EntityId]) -> Self {
        let mut actors: Vec<PxRigidActorRef> = vec![];
        let mut units: Vec<EntityId> = vec![];
        for &entity in entities {
            if let Ok(shape) = world.get_ref(entity, physics_shape()) {
                actors.push(shape.get_actor().unwrap());
            } else if world.has_component(entity, unit_velocity()) {
                units.push(entity);
            }
        }
        Self { actors, units }
    }
    pub fn from_radius(world: &World, position: Vec3, radius: f32) -> Self {
        let scene = world.resource(main_physics_scene());
        let mut hit_call = PxOverlapCallback::new(1000);
        let mut filter_data = PxQueryFilterData::new();
        filter_data.set_flags(PxQueryFlag::DYNAMIC);
        let sphere = PxSphereGeometry::new(radius);
        let mut res = Self::default();
        if scene.overlap(&sphere, PxTransform::from_translation(position), &mut hit_call, &filter_data) {
            let actors: HashSet<_> = hit_call.touches().iter().map(|hit| hit.actor).collect();
            res.actors = actors.into_iter().collect();
        }
        res.units = query((translation(),))
            .incl(unit_velocity())
            .iter(world, None)
            .filter_map(|(id, (&pos,))| if (pos - position).length() <= radius { Some(id) } else { None })
            .collect();
        res
    }
    pub fn apply_force(&self, world: &mut World, get_force: impl Fn(Vec3) -> Vec3) {
        for actor in &self.actors {
            if let Some(actor) = actor.to_rigid_dynamic() {
                let pose = actor.get_global_pose();
                let force = get_force(pose.translation());
                // Kinematic actors can't have force applied to them: https://github.com/OurMachinery/themachinery-public/issues/494
                if !actor.get_rigid_body_flags().contains(PxRigidBodyFlag::KINEMATIC) {
                    actor.add_force(force, None, Some(true));
                }
            }
        }
        for &id in &self.units {
            let mass = world.get(id, unit_mass()).unwrap_or(1.);
            let pos = get_world_position(world, id).unwrap();
            let force = get_force(pos);
            let a = force / mass;
            *world.get_mut(id, unit_velocity()).unwrap() += a * (1. / 60.);
        }
    }
    pub fn apply_force_explosion(&self, world: &mut World, center: Vec3, force: f32, falloff_radius: Option<f32>) {
        let get_force = |pos: Vec3| {
            let mut delta = pos - center;
            if delta.length() == 0. {
                delta = Vec3::X;
            }
            let mut force = delta.normalize() * force;
            if let Some(falloff_radius) = falloff_radius {
                force *= (1. - delta.length() / falloff_radius).max(0.);
            }
            force
        };
        self.apply_force(world, get_force)
    }
}

pub fn apply_force(world: &World, id: EntityId, force: Vec3) -> anyhow::Result<()> {
    let shape = world.get_ref(id, physics_shape())?;
    let actor = shape.get_actor().context("No actor for shape")?;
    let actor = actor.to_rigid_dynamic().context("Not a rigid dynamic")?;
    // Kinematic actors can't have force applied to them: https://github.com/OurMachinery/themachinery-public/issues/494
    if actor.get_rigid_body_flags().contains(PxRigidBodyFlag::KINEMATIC) {
        bail!("Can't apply force to kinematic actor");
    }
    actor.add_force(force, None, Some(true));
    Ok(())
}

pub fn random_position_in_actor(world: &World, id: EntityId) -> Option<Vec3> {
    if world.get_ref(id, physics_shape()).is_ok() {
        // TODO: Do this for real
        let pos = get_world_position(world, id).ok()?;
        return Some(pos + vec3(rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>()) * 2. - 1.);
    }
    None
}

pub fn release_px_scene(scene: PxSceneRef) {
    for actor in scene.get_actors(PxActorTypeFlag::all()) {
        actor.release();
    }
    for constraint in scene.get_constraints() {
        if let Some(joint) = constraint.get_external_reference().to_joint() {
            joint.release();
        } else {
            constraint.release();
        }
    }
    scene.release();
}
