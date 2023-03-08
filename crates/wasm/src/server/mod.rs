use std::sync::Arc;

use ambient_core::asset_cache;
use ambient_ecs::{EntityId, World};
use ambient_input::{player_prev_raw_input, player_raw_input};
use ambient_physics::{helpers::PhysicsObjectCollection, physx::character_controller};
use ambient_std::{
    asset_cache::SyncAssetKeyExt,
    asset_url::{AssetUrl, ServerBaseUrlKey},
    shapes::Ray,
};
use anyhow::Context;
use itertools::Itertools;
use physxx::{PxControllerCollisionFlag, PxControllerFilters};

use crate::shared::{
    bindings::{BindingsBase, BindingsBound, ComponentsParam, WorldRef},
    conversion::{FromBindgen, IntoBindgen},
    implementation, wit, MessageType,
};

pub fn initialize(
    world: &mut World,
    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
) -> anyhow::Result<()> {
    crate::shared::initialize(
        world,
        messenger,
        Bindings {
            base: Default::default(),
            world_ref: Default::default(),
        },
    )?;

    Ok(())
}

#[derive(Clone)]
struct Bindings {
    base: BindingsBase,
    world_ref: WorldRef,
}
impl Bindings {
    pub fn world(&self) -> &World {
        unsafe { self.world_ref.world() }
    }
    pub fn world_mut(&mut self) -> &mut World {
        unsafe { self.world_ref.world_mut() }
    }
}

impl BindingsBound for Bindings {
    fn base(&self) -> &BindingsBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BindingsBase {
        &mut self.base
    }
    fn set_world(&mut self, world: &mut World) {
        unsafe {
            self.world_ref.set_world(world);
        }
    }
    fn clear_world(&mut self) {
        unsafe {
            self.world_ref.clear_world();
        }
    }
}

impl wit::types::Host for Bindings {}
impl wit::entity::Host for Bindings {
    fn spawn(&mut self, data: ComponentsParam) -> anyhow::Result<wit::types::EntityId> {
        implementation::entity::spawn(
            unsafe { self.world_ref.world_mut() },
            &mut self.base.spawned_entities,
            data,
        )
    }

    fn despawn(&mut self, entity: wit::types::EntityId) -> anyhow::Result<bool> {
        implementation::entity::despawn(
            unsafe { self.world_ref.world_mut() },
            &mut self.base.spawned_entities,
            entity,
        )
    }

    fn set_animation_controller(
        &mut self,
        entity: wit::types::EntityId,
        animation_controller: wit::entity::AnimationController,
    ) -> anyhow::Result<()> {
        implementation::entity::set_animation_controller(
            self.world_mut(),
            entity,
            animation_controller,
        )
    }

    fn exists(&mut self, entity: wit::types::EntityId) -> anyhow::Result<bool> {
        implementation::entity::exists(self.world(), entity)
    }

    fn resources(&mut self) -> anyhow::Result<wit::types::EntityId> {
        implementation::entity::resources(self.world())
    }

    fn in_area(
        &mut self,
        position: wit::types::Vec3,
        radius: f32,
    ) -> anyhow::Result<Vec<wit::types::EntityId>> {
        implementation::entity::in_area(self.world_mut(), position, radius)
    }

    fn get_all(&mut self, index: u32) -> anyhow::Result<Vec<wit::types::EntityId>> {
        implementation::entity::get_all(self.world_mut(), index)
    }
}
impl wit::component::Host for Bindings {
    fn get_index(&mut self, id: String) -> anyhow::Result<Option<u32>> {
        implementation::component::get_index(id)
    }

    fn get_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> anyhow::Result<Option<wit::component::ValueResult>> {
        implementation::component::get_component(self.world(), entity, index)
    }

    fn add_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::ValueResult,
    ) -> anyhow::Result<()> {
        implementation::component::add_component(self.world_mut(), entity, index, value)
    }

    fn add_components(
        &mut self,
        entity: wit::types::EntityId,
        data: wit::entity::EntityData,
    ) -> anyhow::Result<()> {
        implementation::component::add_components(self.world_mut(), entity, data)
    }

    fn set_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::ValueResult,
    ) -> anyhow::Result<()> {
        implementation::component::set_component(self.world_mut(), entity, index, value)
    }

    fn set_components(
        &mut self,
        entity: wit::types::EntityId,
        data: ComponentsParam,
    ) -> anyhow::Result<()> {
        implementation::component::set_components(self.world_mut(), entity, data)
    }

    fn has_component(&mut self, entity: wit::types::EntityId, index: u32) -> anyhow::Result<bool> {
        implementation::component::has_component(self.world(), entity, index)
    }

    fn has_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> anyhow::Result<bool> {
        implementation::component::has_components(self.world(), entity, components)
    }

    fn remove_component(&mut self, entity: wit::types::EntityId, index: u32) -> anyhow::Result<()> {
        implementation::component::remove_component(self.world_mut(), entity, index)
    }

    fn remove_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> anyhow::Result<()> {
        implementation::component::remove_components(self.world_mut(), entity, components)
    }

    fn query(
        &mut self,
        query: wit::component::QueryBuild,
        query_event: wit::component::QueryEvent,
    ) -> anyhow::Result<u64> {
        implementation::component::query(&mut self.base.query_states, query, query_event)
    }

    fn query_eval(
        &mut self,
        query_index: u64,
    ) -> anyhow::Result<Vec<(wit::types::EntityId, Vec<wit::component::ValueResult>)>> {
        implementation::component::query_eval(
            unsafe { self.world_ref.world() },
            &mut self.base.query_states,
            query_index,
        )
    }
}
impl wit::player::Host for Bindings {
    fn get_raw_input(
        &mut self,
        player: wit::types::EntityId,
    ) -> anyhow::Result<Option<wit::player::RawInput>> {
        Ok(self
            .world()
            .get_cloned(player.from_bindgen(), player_raw_input())
            .ok()
            .into_bindgen())
    }

    fn get_prev_raw_input(
        &mut self,
        player: wit::types::EntityId,
    ) -> anyhow::Result<Option<wit::player::RawInput>> {
        Ok(self
            .world()
            .get_cloned(player.from_bindgen(), player_prev_raw_input())
            .ok()
            .into_bindgen())
    }
}
impl wit::physics::Host for Bindings {
    fn apply_force(
        &mut self,
        entities: Vec<wit::types::EntityId>,
        force: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        let collection = PhysicsObjectCollection::from_entities(
            self.world_mut(),
            &entities.iter().map(|id| id.from_bindgen()).collect_vec(),
        );
        collection.apply_force(self.world_mut(), |_| force.from_bindgen());
        Ok(())
    }

    fn explode_bomb(
        &mut self,
        position: wit::types::Vec3,
        force: f32,
        radius: f32,
        falloff_radius: Option<f32>,
    ) -> anyhow::Result<()> {
        let position = position.from_bindgen();
        ambient_physics::helpers::PhysicsObjectCollection::from_radius(
            self.world_mut(),
            position,
            radius,
        )
        .apply_force_explosion(self.world_mut(), position, force, falloff_radius);
        Ok(())
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
    ) -> anyhow::Result<wit::physics::CharacterCollision> {
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
                Ok(wit::physics::CharacterCollision {
                    side: res.contains(PxControllerCollisionFlag::CollisionSides),
                    up: res.contains(PxControllerCollisionFlag::CollisionUp),
                    down: res.contains(PxControllerCollisionFlag::CollisionDown),
                })
            }
            Err(_) => Ok(wit::physics::CharacterCollision {
                side: false,
                up: false,
                down: false,
            }),
        }
    }
}
impl wit::event::Host for Bindings {
    fn subscribe(&mut self, name: String) -> anyhow::Result<()> {
        implementation::event::subscribe(&mut self.base.subscribed_events, name)
    }

    fn send(&mut self, name: String, data: ComponentsParam) -> anyhow::Result<()> {
        implementation::event::send(
            self.world_mut(),
            name,
            implementation::component::convert_components_to_entity_data(data),
        )
    }
}
impl wit::asset::Host for Bindings {
    fn url(&mut self, path: String) -> anyhow::Result<Option<String>> {
        let base_url = ServerBaseUrlKey.get(self.world().resource(asset_cache()));
        Ok(Some(AssetUrl::parse(path)?.resolve(&base_url)?.to_string()))
    }
}
