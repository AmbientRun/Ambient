//! Used to implement all the server-specific host functions.
//!
//! If implementing a trait that is also available on the client, it should go in [super].

use ambient_core::player::{player, user_id};
use ambient_ecs::{query, EntityId, World};
use ambient_network::server::player_connection;
use ambient_physics::physx::character_controller;
use ambient_std::shapes::Ray;
use anyhow::Context;
use physxx::{PxControllerCollisionFlag, PxControllerFilters};

use super::super::Bindings;
use crate::shared::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::message,
    wit,
};

impl wit::server_physics::Host for Bindings {
    fn add_force(
        &mut self,
        entity: wit::types::EntityId,
        force: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        let _ = ambient_physics::helpers::add_force(
            self.world_mut(),
            entity.from_bindgen(),
            force.from_bindgen(),
            Some(physxx::PxForceMode::Force),
        );
        Ok(())
    }

    fn add_impulse(
        &mut self,
        entity: wit::types::EntityId,
        force: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        let _ = ambient_physics::helpers::add_force(
            self.world_mut(),
            entity.from_bindgen(),
            force.from_bindgen(),
            Some(physxx::PxForceMode::Impulse),
        );
        Ok(())
    }

    fn add_radial_impulse(
        &mut self,
        position: wit::types::Vec3,
        impulse: f32,
        radius: f32,
        falloff_radius: Option<f32>,
    ) -> anyhow::Result<()> {
        let position = position.from_bindgen();
        ambient_physics::helpers::PhysicsObjectCollection::from_radius(
            self.world_mut(),
            position,
            radius,
        )
        .add_radial_impulse(self.world_mut(), position, impulse, falloff_radius);
        Ok(())
    }

    fn add_force_at_position(
        &mut self,
        entity: wit::types::EntityId,
        force: wit::types::Vec3,
        position: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        let _ = ambient_physics::helpers::add_force_at_position(
            self.world_mut(),
            entity.from_bindgen(),
            force.from_bindgen(),
            position.from_bindgen(),
            Some(physxx::PxForceMode::Force),
        );
        Ok(())
    }

    fn add_impulse_at_position(
        &mut self,
        entity: wit::types::EntityId,
        force: wit::types::Vec3,
        position: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        let _ = ambient_physics::helpers::add_force_at_position(
            self.world_mut(),
            entity.from_bindgen(),
            force.from_bindgen(),
            position.from_bindgen(),
            Some(physxx::PxForceMode::Impulse),
        );
        Ok(())
    }

    fn get_velocity_at_position(
        &mut self,
        entity: wit::types::EntityId,
        position: wit::types::Vec3,
    ) -> anyhow::Result<wit::types::Vec3> {
        let mut result = glam::Vec3::default();
        if let Ok(velocity) = ambient_physics::helpers::get_velocity_at_position(
            self.world_mut(),
            entity.from_bindgen(),
            position.from_bindgen(),
        ) {
            result = velocity;
        }
        Ok(result.into_bindgen())
    }

    fn set_gravity(&mut self, gravity: wit::types::Vec3) -> anyhow::Result<()> {
        self.world_mut()
            .resource(ambient_physics::main_physics_scene())
            .set_gravity(gravity.from_bindgen());
        Ok(())
    }

    fn unfreeze(&mut self, entity: wit::types::EntityId) -> anyhow::Result<()> {
        ambient_physics::helpers::convert_rigid_static_to_dynamic(
            self.world_mut(),
            entity.from_bindgen(),
        );
        Ok(())
    }

    fn freeze(&mut self, entity: wit::types::EntityId) -> anyhow::Result<()> {
        ambient_physics::helpers::convert_rigid_dynamic_to_static(
            self.world_mut(),
            entity.from_bindgen(),
        );
        Ok(())
    }

    fn start_motor(&mut self, entity: wit::types::EntityId, velocity: f32) -> anyhow::Result<()> {
        let joint = ambient_physics::helpers::get_entity_revolute_joint(
            self.world_mut(),
            entity.from_bindgen(),
        )
        .context("Entity doesn't have a motor")?;
        joint.set_drive_velocity(velocity, true);
        joint.set_revolute_flag(physxx::PxRevoluteJointFlag::DRIVE_ENABLED, true);

        Ok(())
    }

    fn stop_motor(&mut self, entity: wit::types::EntityId) -> anyhow::Result<()> {
        let joint = ambient_physics::helpers::get_entity_revolute_joint(
            self.world_mut(),
            entity.from_bindgen(),
        )
        .context("Entity doesn't have a motor")?;
        joint.set_revolute_flag(physxx::PxRevoluteJointFlag::DRIVE_ENABLED, false);

        Ok(())
    }

    fn create_revolute_joint(
        &mut self,
        entity0: wit::types::EntityId,
        transform0: wit::types::Mat4,
        entity1: wit::types::EntityId,
        transform1: wit::types::Mat4,
    ) -> anyhow::Result<()> {
        ambient_physics::helpers::create_revolute_joint(
            self.world_mut(),
            entity0.from_bindgen(),
            transform0.from_bindgen(),
            entity1.from_bindgen(),
            transform1.from_bindgen(),
        )
    }

    fn raycast_first(
        &mut self,
        origin: wit::types::Vec3,
        direction: wit::types::Vec3,
    ) -> anyhow::Result<Option<(wit::types::EntityId, f32)>> {
        let result = ambient_physics::intersection::raycast_first(
            self.world(),
            Ray::new(origin.from_bindgen(), direction.from_bindgen()),
        )
        .map(|t| (t.0.into_bindgen(), t.1.into_bindgen()));

        Ok(result)
    }

    fn raycast(
        &mut self,
        origin: wit::types::Vec3,
        direction: wit::types::Vec3,
    ) -> anyhow::Result<Vec<(wit::types::EntityId, f32)>> {
        let result = ambient_physics::intersection::raycast(
            self.world(),
            Ray::new(origin.from_bindgen(), direction.from_bindgen()),
        )
        .into_iter()
        .map(|t| (t.0.into_bindgen(), t.1.into_bindgen()))
        .collect();

        Ok(result)
    }

    fn move_character(
        &mut self,
        entity: wit::types::EntityId,
        displacement: wit::types::Vec3,
        min_dist: f32,
        elapsed_time: f32,
    ) -> anyhow::Result<wit::server_physics::CharacterCollision> {
        match self
            .world()
            .get(entity.from_bindgen(), character_controller())
        {
            Ok(controller) => {
                let res = controller.move_controller(
                    displacement.from_bindgen(),
                    min_dist,
                    elapsed_time,
                    &PxControllerFilters::new(),
                    None,
                );
                Ok(wit::server_physics::CharacterCollision {
                    side: res.contains(PxControllerCollisionFlag::CollisionSides),
                    up: res.contains(PxControllerCollisionFlag::CollisionUp),
                    down: res.contains(PxControllerCollisionFlag::CollisionDown),
                })
            }
            Err(_) => Ok(wit::server_physics::CharacterCollision {
                side: false,
                up: false,
                down: false,
            }),
        }
    }

    fn set_character_position(
        &mut self,
        entity: wit::types::EntityId,
        position: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        self
            .world()
            .get(entity.from_bindgen(), character_controller())?.set_position(position.from_bindgen().as_dvec3());
        Ok(())
    }


    fn set_character_foot_position(
        &mut self,
        entity: wit::types::EntityId,
        position: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        self
            .world()
            .get(entity.from_bindgen(), character_controller())?.set_foot_position(position.from_bindgen().as_dvec3());
        Ok(())
    }

}

impl wit::server_message::Host for Bindings {
    fn send(
        &mut self,
        target: wit::server_message::Target,
        name: String,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        use wit::server_message::Target;
        let module_id = self.id;
        let world = self.world_mut();

        match target {
            Target::ClientBroadcastUnreliable => {
                send_networked(world, None, module_id, name, data, false)
            }
            Target::ClientBroadcastReliable => {
                send_networked(world, None, module_id, name, data, true)
            }
            Target::ClientTargetedUnreliable(user_id) => {
                send_networked(world, Some(user_id), module_id, name, data, false)
            }
            Target::ClientTargetedReliable(user_id) => {
                send_networked(world, Some(user_id), module_id, name, data, true)
            }
            Target::LocalBroadcast => message::send_local(world, module_id, None, name, data),
            Target::Local(id) => {
                message::send_local(world, module_id, Some(id.from_bindgen()), name, data)
            }
        }
    }
}

fn send_networked(
    world: &World,
    target_user_id: Option<String>,
    module_id: EntityId,
    name: String,
    data: Vec<u8>,
    reliable: bool,
) -> anyhow::Result<()> {
    let connections: Vec<_> = query((user_id(), player_connection()))
        .incl(player())
        .iter(world, None)
        .filter(|(_, (uid, _))| {
            target_user_id
                .as_ref()
                .map(|target_uid| target_uid == *uid)
                .unwrap_or(true)
        })
        .map(|(_, (_, connection))| connection.clone())
        .collect();

    for conn in connections {
        message::send_networked(world, conn, module_id, &name, &data, reliable)?;
    }

    Ok(())
}
