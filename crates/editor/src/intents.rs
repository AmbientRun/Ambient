use anyhow::Context;
use glam::{Mat4, Quat, Vec3, Vec3Swizzles};
use itertools::{izip, process_results, Itertools};
use kiwi_core::{self, selectable, snap_to_ground, transform::get_world_transform};
use kiwi_ecs::{components, uid, uid_lookup, EntityId, EntityUid, World};
use kiwi_intent::{use_old_state, IntentContext, IntentRegistry};
use kiwi_network::get_player_by_user_id;
use kiwi_object::{MultiEntityUID, SpawnConfig};
use kiwi_physics::{
    collider::collider_shapes_convex,
    helpers::{transform_entity, transform_entity_parts},
    main_physics_scene,
    physx::rigid_actor,
    PxShapeUserData,
};
use kiwi_std::{
    log_result,
    shapes::{Ray, Shape, AABB},
};
use kiwi_terrain::get_terrain_height;
use ordered_float::OrderedFloat;
use physxx::{PxActor, PxQueryFilterData, PxRaycastCallback, PxTransform, PxUserData};
use serde::{Deserialize, Serialize};

use crate::{selection, ui::entity_editor::ObjectComponentChange, Selection};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntentTransformRevert {
    uid: EntityUid,
    transform: Mat4,
    snap_to_ground: Option<f32>,
}

fn undo_transform(ctx: IntentContext, undo_state: Vec<IntentTransformRevert>) -> anyhow::Result<()> {
    let world = ctx.world;
    for state in undo_state {
        let id = world.resource(uid_lookup()).get(&state.uid).unwrap();

        if let Some(old_snap_to_ground) = state.snap_to_ground {
            world.add_component(id, snap_to_ground(), old_snap_to_ground).expect("Invalid entity");
        } else {
            world.remove_component(id, snap_to_ground()).expect("Invalid entity")
        }

        transform_entity(world, id, state.transform, false).ok();
    }

    Ok(())
}

components!("editor", {
    /// Moves many entities collectively to another point, while keeping their relative positions
    /// to each other
    intent_translate: IntentTranslate,
    intent_translate_undo: Vec<IntentTransformRevert>,
    intent_place_ray: IntentPlaceRay,
    intent_place_ray_undo: Vec<IntentTransformRevert>,
    intent_set_transform: IntentTransform,
    intent_set_transform_undo: Vec<IntentTransformRevert>,
    intent_reset_terrain_offset: (Vec<EntityUid>, f32),
    intent_reset_terrain_offset_undo: Vec<(EntityUid, Option<f32>)>,
    intent_select: (Selection, SelectMode),
    intent_select_undo: Selection,
    intent_spawn_object_undo: (Vec<EntityUid>, bool, Selection),
    intent_spawn_object2: IntentSpawnObject2,
    intent_duplicate: IntentDuplicate,
    intent_duplicate_undo: Vec<EntityUid>,
    intent_delete: Vec<EntityUid>,
    intent_delete_undo: (World, Selection),
    intent_component_change: (EntityUid, ObjectComponentChange),
    intent_component_change_undo: (EntityUid, ObjectComponentChange),
});

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntentTransform {
    pub entities: Vec<EntityUid>,
    pub transforms: Vec<Mat4>,
    /// If None, use the height after the transform
    pub terrain_offset: TerrainOffset,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntentDuplicate {
    pub entities: Vec<EntityUid>,
    pub new_uids: Vec<EntityUid>,
    pub select: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntentSpawnObject {
    pub object_id: String,
    pub entity_uid: EntityUid,
    pub position: Vec3,
    pub select: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntentSpawnObject2 {
    pub object_url: String,
    pub entity_uid: MultiEntityUID,
    pub position: Vec3,
    pub select: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MovePosition {
    Raycast { ray: Ray },
    Position { position: Vec3 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnappingShape {
    Edge { dir: Vec3 },
    Surface { tangent: Vec3, bitangent: Vec3 },
    Volume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapping {
    pub size: f32,
    pub origin: Vec3,
    pub mode: SnappingShape,
}

impl Snapping {
    /// Performs the snapping
    pub fn snap(&self, point: Vec3) -> Vec3 {
        match self.mode {
            SnappingShape::Edge { dir } => {
                debug_assert!(dir.is_normalized(), "Dir: {dir}");
                let p = ((point - self.origin).dot(dir) / self.size).round() * self.size;
                self.origin + dir * p
            }
            SnappingShape::Surface { tangent, bitangent } => {
                assert!(tangent.is_normalized());
                assert!(bitangent.is_normalized());
                let point = point - self.origin;

                assert!(tangent.dot(bitangent).abs() < 0.001, "Surface tangent and bitangent are not orthogonal {tangent} x {bitangent}",);

                let l = (point.dot(tangent) / self.size).round() * self.size;
                let r = (point.dot(bitangent) / self.size).round() * self.size;

                self.origin + tangent * l + bitangent * r
            }
            SnappingShape::Volume => (point / self.size).round() * self.size,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentTranslate {
    pub targets: Vec<EntityUid>,
    pub position: Vec3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPlaceRay {
    pub targets: Vec<EntityUid>,
    pub ray: Ray,
    /// Apply snapping relative to the object the ray intersected
    pub snap: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum SelectMode {
    Set,
    Add,
    Remove,
    Clear,
}
impl Default for SelectMode {
    fn default() -> Self {
        Self::Set
    }
}

fn axis_aligned_plane(normal: Vec3) -> (Vec3, Vec3) {
    assert!(normal.is_normalized(), "Normal is not normalized");
    if normal.dot(Vec3::Z).abs() < 0.99 {
        let tangent = normal.cross(Vec3::Z).normalize_or_zero();
        let bitangent = tangent.cross(normal).normalize_or_zero();
        (tangent, bitangent)
    } else {
        let tangent = normal.cross(Vec3::X).normalize_or_zero();
        let bitangent = tangent.cross(normal).normalize_or_zero();
        (tangent, bitangent)
    }
}

#[profiling::function]
fn resolve_clipping(world: &mut World, entities: &[EntityUid], ids: &[EntityId], source: Vec3, target: Intersection) -> Option<Vec3> {
    let scene = world.resource(main_physics_scene());

    let total_bounds: AABB = ids.iter().flat_map(|&id| Some(world.get(id, rigid_actor()).ok()?.get_world_bounds(1.0).into())).collect();

    let dir = -target.normal;
    let initial_offset = (total_bounds.support(dir) - total_bounds.center()).project_onto(dir) * 2.1;

    entities
        .iter()
        .zip_eq(ids)
        .flat_map(|(_, &id)| -> Option<_> {
            profiling::scope!("query_intersection");
            // let id = *world.resource(uid_lookup()).get(&entity.id)?;

            let transform = get_world_transform(world, id).ok()?;
            // let convex_geom = world.get_ref(id, collider_shapes_convex()).expect("Missing convex colliders on entity");
            let shapes = world.get_ref(id, collider_shapes_convex()).ok()?;

            let target = &target;
            let ids = &ids;

            Some(shapes.iter().flat_map(move |shape| {
                let local_pose = shape.get_local_pose();
                let local_rot = local_pose.rotation();
                let local_pos = local_pose.translation();
                let start = target.point + local_pos - initial_offset;

                let (_, rot, _) = transform.to_scale_rotation_translation();

                let pose = PxTransform::new(start, rot * local_rot);

                let filter = PxQueryFilterData::new();
                let sweep = scene.sweep(&shape.get_geometry(), &pose, dir, 1024.0, filter);
                let sweep = sweep
                    .touches()
                    .into_iter()
                    .filter(|v| {
                        let id = v.shape.as_ref().unwrap().get_user_data::<PxShapeUserData>().unwrap().entity;
                        !ids.contains(&id)
                    })
                    .min_by_key(|v| OrderedFloat(v.distance))?;

                let (_, _, pos) = transform.to_scale_rotation_translation();
                let relative_offset = pos - source;
                tracing::debug!("Relative offset: {relative_offset}, sweep: {sweep:?}");

                let point = start + sweep.distance * dir;
                // tracing::info!(?start, ?point, ?local_pos, ?sweep, "Found sweep result");

                let offset = (point - rot * local_pos + relative_offset) - target.point;
                Some(offset)
            }))
        })
        .flatten()
        // Find the support point which is furthest in the wanted direction
        .max_by_key(|v| ordered_float::NotNan::new(-v.dot(dir)).unwrap())
}

pub fn register_intents(reg: &mut IntentRegistry) {
    reg.register(
        intent_place_ray(),
        intent_place_ray_undo(),
        |ctx, IntentPlaceRay { targets, ray, snap }| {
            profiling::scope!("handle_intent_move");
            let world = ctx.world;

            enum SurfaceOffset {
                Keep { _normal: Vec3 },
                Update,
            }

            let (ids, transforms): (Vec<_>, Vec<_>) = process_results(
                targets.iter().map(|uid| -> anyhow::Result<_> {
                    let id = world.resource(uid_lookup()).get(uid).context("Failed to resolve uid")?;

                    let transform = get_world_transform(world, id).with_context(|| format!("Failed to get world transform for {uid:?}"))?;

                    Ok((id, transform))
                }),
                |iter| iter.unzip(),
            )?;

            let midpoint = transforms
                .iter()
                .map(|v| {
                    let (_, _, pos) = v.to_scale_rotation_translation();
                    pos
                })
                .fold(Vec3::ZERO, |acc, x| acc + x)
                / transforms.len().max(1) as f32;

            profiling::scope!("intent_move");
            // tracing::info!("Bounding box: {bounds:?}");

            let intersect = find_world_intersection_without_entities(world, ray, &ids, 500.);

            let target = if let Some(mut intersect) = intersect {
                use kiwi_terrain::terrain_world_cell;

                // The terrain should always offset upwards
                if world.get(intersect.id, terrain_world_cell()).is_ok() {
                    intersect.normal = Vec3::Z
                }

                let subject_transform = get_world_transform(world, intersect.id).expect("Missing position for entity");
                let (_, _, subject_pos) = subject_transform.to_scale_rotation_translation();

                // log::info!("Snap: {snap:?}");
                let target = match snap {
                    None => intersect.point,
                    // (Some(size), true) => Snapping { size, origin: Vec3::ZERO, mode: SnappingShape::Volume }.snap(intersect.point),
                    Some(snap) => {
                        let (tangent, bitangent) = axis_aligned_plane(intersect.normal);
                        Snapping {
                            size: snap,
                            origin: subject_pos + (intersect.point - subject_pos).project_onto(intersect.normal),
                            mode: SnappingShape::Surface { tangent, bitangent },
                        }
                        .snap(intersect.point)
                    }
                };

                // Once the snapped intersection point has been
                // established, move out to clip to the side of
                // the manipulated objects
                let clip = resolve_clipping(world, &targets, &ids, midpoint, intersect).unwrap_or_default();

                target + clip
            } else {
                ray.origin + ray.dir * 100.0
            };

            izip!(targets, ids, transforms)
                .map(|(uid, id, transform): (_, _, Mat4)| {
                    {
                        let old_snap_to_ground = world.get(id, snap_to_ground()).ok();

                        let (scale, rot, pos) = transform.to_scale_rotation_translation();

                        // World space position
                        let new_pos = pos - midpoint + target;
                        tracing::debug!(?midpoint, "Moving {uid} {pos} => {new_pos}");

                        update_snap_to_ground(world, id, pos);

                        let res = transform_entity_parts(world, id, new_pos, rot, scale).context("Failed to transform entity {id}");

                        log_result!(res);

                        Ok(IntentTransformRevert { snap_to_ground: old_snap_to_ground, transform, uid })
                    }
                })
                .collect()
        },
        undo_transform,
        use_old_state,
    );

    reg.register(
        intent_translate(),
        intent_translate_undo(),
        |ctx, IntentTranslate { targets, position }| {
            profiling::scope!("handle_intent_move");
            let world = ctx.world;

            enum SurfaceOffset {
                Keep { _normal: Vec3 },
                Update,
            }

            let (ids, transforms): (Vec<_>, Vec<_>) = process_results(
                targets.iter().map(|uid| -> anyhow::Result<_> {
                    let id = world.resource(uid_lookup()).get(uid).context("Failed to resolve uid")?;

                    let transform = get_world_transform(world, id).with_context(|| format!("Failed to get world transform for {uid:?}"))?;

                    Ok((id, transform))
                }),
                |iter| iter.unzip(),
            )?;

            let midpoint = transforms
                .iter()
                .map(|v| {
                    let (_, _, pos) = v.to_scale_rotation_translation();
                    pos
                })
                .fold(Vec3::ZERO, |acc, x| acc + x)
                / transforms.len().max(1) as f32;

            izip!(targets, ids, transforms)
                .map(|(uid, id, transform): (_, _, Mat4)| {
                    let old_snap_to_ground = world.get(id, snap_to_ground()).ok();

                    let (scale, rot, pos) = transform.to_scale_rotation_translation();

                    // World space position
                    let new_pos = pos - midpoint + position;
                    tracing::debug!(?midpoint, "Moving {uid} {pos} => {new_pos}");

                    update_snap_to_ground(world, id, pos);

                    let res = transform_entity_parts(world, id, new_pos, rot, scale).context("Failed to transform entity {id}");

                    log_result!(res);

                    Ok(IntentTransformRevert { snap_to_ground: old_snap_to_ground, transform, uid })
                })
                .collect()
        },
        undo_transform,
        use_old_state,
    );

    reg.register(
        intent_set_transform(),
        intent_set_transform_undo(),
        |ctx, intent| {
            let world = ctx.world;
            intent
                .entities
                .iter()
                .zip_eq(intent.transforms)
                .map(|(uid, transform)| {
                    let id = world.resource(uid_lookup()).get(uid).unwrap();
                    let old_transform = get_world_transform(world, id).context("No transform")?;

                    let (scale, rot, pos) = transform.to_scale_rotation_translation();

                    let old_snap_to_ground = world.get(id, snap_to_ground()).ok();

                    update_snap_to_ground(world, id, pos);

                    transform_entity_parts(world, id, pos, rot, scale).context("Failed to transform entity")?;

                    Ok(IntentTransformRevert { transform: old_transform, snap_to_ground: old_snap_to_ground, uid: uid.clone() })
                })
                .collect::<Result<Vec<_>, _>>()
        },
        |ctx, transforms| {
            let world = ctx.world;
            for old_state in transforms {
                let id = world.resource(uid_lookup()).get(&old_state.uid).unwrap();
                if let Some(old_snap_to_ground) = old_state.snap_to_ground {
                    world.add_component(id, snap_to_ground(), old_snap_to_ground).expect("Invalid entity");
                } else {
                    world.remove_component(id, snap_to_ground()).expect("Invalid entity")
                }

                transform_entity(world, id, old_state.transform, false).ok();
            }
            Ok(())
        },
        use_old_state,
    );

    reg.register(
        intent_reset_terrain_offset(),
        intent_reset_terrain_offset_undo(),
        |ctx, intent| {
            let world = ctx.world;
            intent
                .0
                .iter()
                .map(|uid| {
                    let id = world.resource(uid_lookup()).get(uid).unwrap();

                    let transform = get_world_transform(world, id).context("No transform")?;
                    let (scale, rot, pos) = transform.to_scale_rotation_translation();

                    set_snap_to_ground(world, id, intent.1);

                    let old_snap_to_ground = world.get(id, snap_to_ground()).ok();
                    transform_entity_parts(world, id, pos, rot, scale).context("Failed to transform")?;
                    Ok((uid.clone(), old_snap_to_ground)) as anyhow::Result<_>
                })
                .collect::<Result<Vec<_>, _>>()
        },
        |ctx, old_offset| {
            let world = ctx.world;
            for (id, old_offset) in old_offset {
                let id = world.resource(uid_lookup()).get(&id).unwrap();
                if let Some(old_offset) = old_offset {
                    world.add_component(id, snap_to_ground(), old_offset).expect("Invalid entity");
                } else {
                    world.remove_component(id, snap_to_ground()).expect("Invalid entity")
                }
            }
            Ok(())
        },
        use_old_state,
    );

    reg.register(
        intent_select(),
        intent_select_undo(),
        |ctx, (new_selection, select)| {
            let world = ctx.world;
            let player_entity = get_player_by_user_id(world, ctx.user_id).context("No player with that user_id found")?;
            let selection = world.get_mut(player_entity, selection()).context("Selection missing")?;

            let old_selection = selection.clone();
            match select {
                SelectMode::Set => {
                    *selection = new_selection.clone();
                }
                SelectMode::Add => {
                    selection.union(&new_selection);
                }
                SelectMode::Remove => {
                    selection.difference(&new_selection);
                }
                SelectMode::Clear => {
                    assert!(new_selection.is_empty());
                    selection.clear();
                }
            }

            Ok(old_selection)
        },
        |ctx, prev_selection| {
            let world = ctx.world;
            if let Some(player_entity) = get_player_by_user_id(world, ctx.user_id) {
                world.set(player_entity, selection(), prev_selection).ok();
            }
            Ok(())
        },
        use_old_state,
    );

    reg.register(
        intent_spawn_object2(),
        intent_spawn_object_undo(),
        |ctx, IntentSpawnObject2 { object_url, entity_uid, position, select }| {
            let user_id = ctx.user_id;
            let world = ctx.world;

            let ids = tokio::task::block_in_place(|| {
                let mut conf = SpawnConfig::new(entity_uid.clone(), position, Quat::IDENTITY, Vec3::ONE);
                conf.components.set_self(selectable(), ());
                kiwi_object::spawn_preloaded_by_url(world, object_url, conf)
            })?;
            let uids = ids.iter().map(|id| world.get_ref(*id, uid()).unwrap().clone()).collect_vec();

            let player_entity = get_player_by_user_id(world, user_id).context("Player not found")?;
            let old_selection = world.get_ref(player_entity, selection()).cloned().context("Failed to get selection")?;
            let new_lookups = ids.iter().map(|&id| (world.get_ref(id, uid()).unwrap().clone(), id)).collect_vec();
            world.resource_mut(uid_lookup()).extend(new_lookups);
            // Set the player selection to the spawned object
            if select {
                tracing::debug!("Setting player selection to: {uids:?}");
                world.set(player_entity, selection(), Selection::new(uids.clone())).context("Failed to set selection")?;
            }
            Ok((uids, select, old_selection))
        },
        move |ctx, (uids, select, old_selection)| {
            let user_id = ctx.user_id.to_string();
            let world = ctx.world;
            for uid in uids {
                let id = world.resource_mut(uid_lookup()).remove(&uid).context("No such entity")?;
                world.despawn(id);
            }
            if select {
                if let Some(player_entity) = get_player_by_user_id(world, &user_id) {
                    world.set(player_entity, selection(), old_selection).ok();
                }
            }
            Ok(())
        },
        use_old_state,
    );
    reg.register(
        intent_duplicate(),
        intent_duplicate_undo(),
        |ctx, IntentDuplicate { entities, new_uids, select }| {
            let world = ctx.world;
            let player_entity = get_player_by_user_id(world, ctx.user_id).context("Player not found")?;
            let ids = entities.iter().map(|id| world.resource(uid_lookup()).get(id).unwrap()).collect_vec();
            let storage = World::from_entities(world, ids, true);

            let ids = storage.spawn_into_world(world, None);
            for (id, new_uid) in ids.iter().zip(new_uids.iter()) {
                world.set(*id, uid(), new_uid.clone()).unwrap();
            }

            world.resource_mut(uid_lookup()).extend(new_uids.iter().cloned().zip(ids.into_iter()));

            // Set the selection to the new objects
            if select {
                world.set(player_entity, selection(), Selection::new(new_uids.clone())).ok();
            }

            Ok(new_uids)
        },
        |ctx, ids| {
            let world = ctx.world;
            for id in ids {
                let id = world.resource_mut(uid_lookup()).remove(&id).unwrap();
                world.despawn(id);
            }
            Ok(())
        },
        use_old_state,
    );
    reg.register(
        intent_delete(),
        intent_delete_undo(),
        |ctx, entities| {
            let world = ctx.world;
            let player_entity = get_player_by_user_id(world, ctx.user_id).context("Player not found")?;
            let ids = entities.iter().flat_map(|id| world.resource(uid_lookup()).get(id)).collect_vec();
            let old = World::from_entities(world, ids.clone(), true);

            for &id in ids.iter() {
                let uid = world.get_ref(id, uid()).cloned();
                world.despawn(id);
                if let Ok(uid) = uid {
                    world.resource_mut(uid_lookup()).remove(&uid);
                }
            }

            let old_selection = {
                let to_remove = ids.iter().flat_map(|id| world.get_ref(*id, uid()).cloned()).collect_vec();
                let sel = world.get_mut(player_entity, selection()).unwrap();
                let old_sel = sel.clone();
                sel.difference(&Selection::new(to_remove));
                old_sel
            };
            Ok((old, old_selection))
        },
        |ctx, (entities, old_selection)| {
            let world = ctx.world;
            let ids = entities.spawn_into_world(world, None);
            for id in ids {
                let uid = world.get_ref(id, uid()).cloned();
                if let Ok(uid) = uid {
                    world.resource_mut(uid_lookup()).insert(uid, id);
                }
            }
            if let Some(player_entity) = get_player_by_user_id(world, ctx.user_id) {
                world.set(player_entity, selection(), old_selection).ok();
            }
            Ok(())
        },
        use_old_state,
    );
    reg.register(
        intent_component_change(),
        intent_component_change_undo(),
        |ctx, (uid, change)| {
            let world = ctx.world;
            let id = world.resource(uid_lookup()).get(&uid).unwrap();
            Ok((uid, change.apply_to_entity(world, id)))
        },
        |ctx, (uid, revert)| {
            let world = ctx.world;
            if let Ok(id) = world.resource(uid_lookup()).get(&uid) {
                revert.apply_to_entity(world, id);
            }
            Ok(())
        },
        use_old_state,
    );

    kiwi_terrain::intents::register_intents(reg);
    // Box::new(common_intent_systems()),
    // ],
}

/// Describes a ray intersection
#[derive(Debug, Clone)]
pub struct Intersection {
    /// The hit entity
    pub id: EntityId,
    pub dist: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

/// Perform a ray intersect while excluding some entities
fn find_world_intersection_without_entities(world: &mut World, ray: Ray, entities: &[EntityId], max_dist: f32) -> Option<Intersection> {
    let mut hit = PxRaycastCallback::new(100);
    let scene = world.resource(main_physics_scene());
    let filter_data = PxQueryFilterData::new();
    if scene.raycast(ray.origin, ray.dir, max_dist, &mut hit, None, &filter_data) {
        let min_dist = hit
            .touches()
            .into_iter()
            .filter_map(|hit| {
                if let Some(shape) = hit.shape {
                    let ud = shape.get_user_data::<PxShapeUserData>().unwrap();
                    if !entities.contains(&ud.entity) {
                        return Some((OrderedFloat(hit.distance), hit.normal, hit.position, ud.entity));
                    }
                }
                None
            })
            .min_by_key(|v| v.0);
        if let Some((dist, normal, point, id)) = min_dist {
            if dist.0 < max_dist {
                return Some(Intersection { dist: *dist, point, normal, id });
            }
        }
    }

    None
}

#[derive(Copy, Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub enum TerrainOffset {
    Keep,
    Update,
    Set(f32),
}

fn update_snap_to_ground(world: &mut World, id: EntityId, pos: Vec3) {
    let terrain_height = get_terrain_height(world, pos.xy());
    if let Some(terrain_height) = terrain_height {
        let new_offset = pos.z - terrain_height;

        if let Ok(snap_to_ground) = world.get_mut(id, snap_to_ground()) {
            *snap_to_ground = new_offset;
        }
    }
}
fn set_snap_to_ground(world: &mut World, id: EntityId, height: f32) {
    // Modify the transformed z value
    world.add_component(id, snap_to_ground(), height).expect("Invalid entity");
}
