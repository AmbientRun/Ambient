use ambient_ecs::{ArchetypeFilter, Entity, EntityId};
use ambient_intent::server_push_intent;
use ambient_native_std::{shapes::Ray, unwrap_log_err};
use ambient_network::server::RpcArgs as ServerRpcArgs;
use ambient_physics::visualization::{run_visualize_collider, visualize_collider};
use ambient_physics::{
    helpers::{
        convert_rigid_dynamic_to_static, convert_rigid_static_to_dynamic, unweld_multi, weld_multi,
    },
    intersection::{intersect_frustum, raycast_filtered, rpc_pick, RaycastFilter},
};
use ambient_rpc::RpcRegistry;
use anyhow::Context;
use bitflags::bitflags;
use glam::{vec3, Vec3};
use serde::{Deserialize, Serialize};

use crate::intents::{intent_select, SelectMode};
use crate::Selection;
use ambient_core::selectable;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct AxisFlags: u32 {
        const X = 0b00000001;
        const Y = 0b00000010;
        const Z = 0b00000100;
    }
}

impl AxisFlags {
    pub fn as_vec3(self) -> Vec3 {
        vec3(
            self.contains(AxisFlags::X) as u32 as f32,
            self.contains(AxisFlags::Y) as u32 as f32,
            self.contains(AxisFlags::Z) as u32 as f32,
        )
    }
}

pub fn register_server_rpcs(reg: &mut RpcRegistry<ServerRpcArgs>) {
    reg.register(rpc_pick);
    reg.register(rpc_select);
    reg.register(rpc_weld);
    reg.register(rpc_unweld);
    reg.register(rpc_freeze);
    reg.register(rpc_unfreeze);
    reg.register(rpc_toggle_visualize_colliders);
    // reg.register(rpc_save);
    reg.register(rpc_spawn);
    // reg.register(rpc_teleport_player);
}

pub async fn rpc_select(args: ServerRpcArgs, (method, mode): (SelectMethod, SelectMode)) {
    let entities = {
        let mut state = args.state.lock();
        let world = unwrap_log_err!(state
            .get_player_world_mut(&args.user_id)
            .context("No player world"));
        match method {
            SelectMethod::Frustum(frustum) => intersect_frustum(world, &frustum)
                .into_iter()
                .filter(|id| world.has_component(*id, selectable()))
                .collect(),
            SelectMethod::Ray(ray) => {
                if let Some((entity, _)) = raycast_filtered(
                    world,
                    RaycastFilter {
                        entities: Some(ArchetypeFilter::new().incl(selectable())),
                        collider_type: None,
                    },
                    ray,
                    None,
                ) {
                    Selection::new([entity])
                } else {
                    Default::default()
                }
            }
            SelectMethod::Manual(ids) => ids,
        }
    };

    let collapse_id = format!("{entities:?} {mode:?}");
    server_push_intent(
        args.state,
        intent_select(),
        (entities, mode),
        args.user_id.clone(),
        Some(collapse_id),
    )
    .await;
}

pub async fn rpc_weld(args: ServerRpcArgs, entities: Vec<EntityId>) {
    let mut state = args.state.lock();
    let world = unwrap_log_err!(state
        .get_player_world_mut(&args.user_id)
        .context("No player world"));
    weld_multi(world, entities);
}

pub async fn rpc_unweld(args: ServerRpcArgs, entities: Vec<EntityId>) {
    let mut state = args.state.lock();
    let world = unwrap_log_err!(state
        .get_player_world_mut(&args.user_id)
        .context("No player world"));
    unweld_multi(world, entities);
}

pub async fn rpc_freeze(args: ServerRpcArgs, entities: Vec<EntityId>) {
    let mut state = args.state.lock();
    let world = unwrap_log_err!(state
        .get_player_world_mut(&args.user_id)
        .context("No player world"));
    for entity in entities {
        convert_rigid_dynamic_to_static(world, entity);
    }
}
pub async fn rpc_unfreeze(args: ServerRpcArgs, entities: Vec<EntityId>) {
    let mut state = args.state.lock();
    let world = unwrap_log_err!(state
        .get_player_world_mut(&args.user_id)
        .context("No player world"));
    for entity in entities {
        convert_rigid_static_to_dynamic(world, entity);
    }
}
pub async fn rpc_toggle_visualize_colliders(args: ServerRpcArgs, entities: Vec<EntityId>) {
    let mut state = args.state.lock();
    let world = unwrap_log_err!(state
        .get_player_world_mut(&args.user_id)
        .context("No player world"));

    let enabled = entities
        .iter()
        .any(|&v| world.has_component(v, visualize_collider()));

    if !entities.is_empty() {
        for entity in entities {
            run_visualize_collider(world, entity, !enabled);
        }
    }
}
// pub async fn rpc_save(args: ServerRpcArgs, _: ()) {
//     let (stored_map, map_path) = {
//         let state = args.state.lock();
//         let instance = state.instances.get(MAIN_INSTANCE_ID).unwrap();
//         let stored_map = serde_json::to_vec(&instance.world).unwrap();
//         let map_path = instance.world.resource(map_url()).clone();
//         (stored_map, map_path)
//     };
//     log_warning!(save_world(stored_map, map_path).await);
// }
pub async fn rpc_spawn(args: ServerRpcArgs, entity_data: Entity) -> Option<EntityId> {
    let mut state = args.state.lock();
    let world = state.get_player_world_mut(&args.user_id)?;
    Some(entity_data.spawn(world))
}

// pub async fn rpc_teleport_player(args: ServerRpcArgs, position: Vec3) -> Result<(), ECSError> {
//     let mut state = args.state.lock();
//     let world = state.get_player_world_mut(&args.user_id).ok_or_else(|| ECSError::NoSuchEntity { entity_id: EntityId::null() })?;
//     if let Some(player_id) = args.get_player(world) {
//         let body_id = world.get(player_id, player_body_ref())?;
//         ambient_physics::helpers::transform_entity_parts(
//             world,
//             body_id,
//             position,
//             world.get(body_id, rotation())?,
//             world.get(body_id, scale())?,
//         )?;
//     }

//     Ok(())
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SelectMethod {
    Frustum([Vec3; 8]),
    Ray(Ray),
    Manual(Selection),
}
