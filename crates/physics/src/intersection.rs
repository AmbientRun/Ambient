use std::collections::HashSet;

use elements_core::{asset_cache, transform::translation};
use elements_ecs::{query, ArchetypeFilter, EntityId, World};
use elements_meshes::cuboid::CuboidMesh;
use elements_network::client::GameRpcArgs;
use elements_std::{asset_cache::SyncAssetKeyExt, mesh::Mesh, shapes::Ray};
use glam::Vec3;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use physxx::{
    PxConvexFlag, PxConvexMesh, PxConvexMeshDesc, PxConvexMeshGeometry, PxOverlapCallback, PxQueryFilterData, PxRaycastCallback, PxRigidActor, PxShape, PxTransform, PxUserData
};
use serde::{Deserialize, Serialize};

use crate::{main_physics_scene, physx::PhysicsKey, ColliderScene, PxShapeUserData};

pub fn get_entities_in_radius(world: &World, center: Vec3, radius: f32) -> Vec<EntityId> {
    query((translation(),))
        .iter(world, None)
        .filter_map(|(id, (&pos,))| if (pos - center).length() <= radius { Some(id) } else { None })
        .collect_vec()
}

pub fn raycast_first(world: &World, ray: Ray) -> Option<(EntityId, f32)> {
    raycast_first_px(world, ray).and_then(|(shape, dist)| shape.get_user_data::<PxShapeUserData>().map(|ud| (ud.entity, dist)))
}

fn raycast_first_px(world: &World, ray: Ray) -> Option<(PxShape, f32)> {
    (0..3)
        .filter_map(|i| raycast_first_collider_type_px(world, ColliderScene::from_usize(i), ray))
        .sorted_by_key(|x| OrderedFloat(x.1))
        .next()
}

pub fn raycast_first_collider_type(world: &World, collider_type: ColliderScene, ray: Ray) -> Option<(EntityId, f32)> {
    raycast_first_collider_type_px(world, collider_type, ray)
        .and_then(|(shape, dist)| shape.get_user_data::<PxShapeUserData>().map(|ud| (ud.entity, dist)))
}
pub fn raycast_first_collider_type_px(world: &World, collider_type: ColliderScene, ray: Ray) -> Option<(PxShape, f32)> {
    let mut hit = PxRaycastCallback::new(0);
    let scene = collider_type.get_scene(world);
    let filter_data = PxQueryFilterData::new();
    if scene.raycast(ray.origin, ray.dir, f32::MAX, &mut hit, None, &filter_data) {
        let block = hit.block().unwrap();
        if let Some(shape) = block.shape {
            return Some((shape, block.distance));
        }
    }
    None
}

pub fn raycast(world: &World, ray: Ray) -> Vec<(EntityId, f32)> {
    raycast_px(world, ray)
        .into_iter()
        .flat_map(|(shape, dist)| shape.get_user_data::<PxShapeUserData>().map(|ud| (ud.entity, dist)))
        .collect_vec()
}

fn raycast_px(world: &World, ray: Ray) -> Vec<(PxShape, f32)> {
    (0..3)
        .flat_map(|i| raycast_collider_type_px(world, ColliderScene::from_usize(i), ray).into_iter())
        .sorted_by_key(|x| OrderedFloat(x.1))
        .collect_vec()
}

pub fn raycast_collider_type(world: &World, collider_type: ColliderScene, ray: Ray) -> Vec<(EntityId, f32)> {
    raycast_collider_type_px(world, collider_type, ray)
        .into_iter()
        .filter_map(|(shape, dist)| shape.get_user_data::<PxShapeUserData>().map(|ud| (ud.entity, dist)))
        .collect()
}
pub fn raycast_collider_type_px(world: &World, collider_type: ColliderScene, ray: Ray) -> Vec<(PxShape, f32)> {
    let mut hit = PxRaycastCallback::new(100);
    let scene = collider_type.get_scene(world);
    let filter_data = PxQueryFilterData::new();
    if scene.raycast(ray.origin, ray.dir, f32::MAX, &mut hit, None, &filter_data) {
        return hit.touches().into_iter().filter_map(|hit| hit.shape.map(|shape| (shape, hit.distance))).collect_vec();
    }
    Vec::new()
}

pub fn intersect_frustum(world: &World, frustum_corners: &[Vec3; 8]) -> Vec<EntityId> {
    let mut hit_call = PxOverlapCallback::new(1000);
    let filter_data = PxQueryFilterData::new();
    let mesh = Mesh::from(&CuboidMesh { positions: *frustum_corners, color: None, texcoords: false, normals: false, tangents: false });
    let physics = PhysicsKey.get(world.resource(asset_cache()));
    let desc = PxConvexMeshDesc {
        points: mesh.positions.unwrap(),
        indices: mesh.indices,
        vertex_limit: None,
        flags: Some(PxConvexFlag::COMPUTE_CONVEX),
    };
    let px_mesh = match PxConvexMesh::from_desc(physics.physics, physics.cooking, desc) {
        Ok(px_mesh) => px_mesh,
        Err(err) => {
            tracing::warn!("Failed to construct interesection frustum: {:?}", err);
            return Vec::new();
        }
    };
    let geo = PxConvexMeshGeometry::new(&px_mesh, None, None);

    let scene = world.resource(main_physics_scene());
    if scene.overlap(&geo, PxTransform::identity(), &mut hit_call, &filter_data) {
        let mut res = HashSet::new();
        for hit in hit_call.touches() {
            for shape in hit.actor.get_shapes() {
                let ud = shape.get_user_data::<PxShapeUserData>().unwrap();
                res.insert(ud.entity);
            }
        }
        res.into_iter().collect()
    } else {
        Vec::new()
    }
}

pub async fn rpc_pick(args: GameRpcArgs, (ray, filter): (Ray, RaycastFilter)) -> Option<(EntityId, f32)> {
    let state = args.state.lock();
    raycast_filtered(state.get_player_world(&args.user_id)?, filter, ray)
}

pub fn raycast_filtered(world: &World, filter: RaycastFilter, ray: Ray) -> Option<(EntityId, f32)> {
    let hits =
        if let Some(collider_type) = filter.collider_type { raycast_collider_type(world, collider_type, ray) } else { raycast(world, ray) };
    if let Some(filter) = &filter.entities {
        hits.into_iter().filter(|(id, _)| filter.matches_entity(world, *id)).min_by_key(|(_, dist)| OrderedFloat(*dist))
    } else {
        hits.into_iter().min_by_key(|(_, dist)| OrderedFloat(*dist))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaycastFilter {
    pub entities: Option<ArchetypeFilter>,
    pub collider_type: Option<ColliderScene>,
}
