use std::sync::Arc;

use ambient_core::asset_cache;
use ambient_ecs::{with_component_registry, ComponentSet, EntityId, QueryEvent, World};
use ambient_physics::{helpers::PhysicsObjectCollection, physx::character_controller};
use ambient_std::{
    asset_cache::SyncAssetKeyExt,
    asset_url::{AssetUrl, ServerBaseUrlKey},
};
use anyhow::Context;
use itertools::Itertools;
use physxx::{PxControllerCollisionFlag, PxControllerFilters};

use crate::shared::{
    bindings::{
        add_component, convert_components_to_entity_data, read_component_from_world,
        read_primitive_component_from_entity_accessor, set_component, BindingsBase, BindingsBound,
        ComponentsParam,
    },
    conversion::{FromBindgen, IntoBindgen},
    implementation as shared_impl, wit, MessageType,
};
mod implementation;
use implementation as server_impl;

#[derive(Clone)]
struct Bindings {
    base: BindingsBase,
}
impl Bindings {
    pub fn world(&self) -> &World {
        self.base.world()
    }
    pub fn world_mut(&mut self) -> &mut World {
        self.base.world_mut()
    }
}

impl wit::types::Types for Bindings {}
impl wit::entity::Entity for Bindings {
    fn spawn(&mut self, data: ComponentsParam) -> anyhow::Result<wit::types::EntityId> {
        let id =
            shared_impl::entity::spawn(self.world_mut(), convert_components_to_entity_data(data));

        self.base.spawned_entities.insert(id);
        Ok(id.into_bindgen())
    }

    fn despawn(&mut self, entity: wit::types::EntityId) -> anyhow::Result<bool> {
        let entity = entity.from_bindgen();
        let despawn = shared_impl::entity::despawn(self.world_mut(), entity);
        Ok(if let Some(id) = despawn {
            self.base.spawned_entities.remove(&id);
            true
        } else {
            false
        })
    }

    fn set_animation_controller(
        &mut self,
        entity: wit::types::EntityId,
        animation_controller: wit::entity::AnimationController,
    ) -> anyhow::Result<()> {
        shared_impl::entity::set_animation_controller(
            self.world_mut(),
            entity.from_bindgen(),
            animation_controller.from_bindgen(),
        )
    }

    fn exists(&mut self, entity: wit::types::EntityId) -> anyhow::Result<bool> {
        Ok(self.world().exists(entity.from_bindgen()))
    }

    fn resources(&mut self) -> anyhow::Result<wit::types::EntityId> {
        Ok(shared_impl::entity::resources(self.world()).into_bindgen())
    }

    fn in_area(
        &mut self,
        position: wit::types::Vec3,
        radius: f32,
    ) -> anyhow::Result<Vec<wit::types::EntityId>> {
        Ok(
            shared_impl::entity::in_area(self.world_mut(), position.from_bindgen(), radius)?
                .into_bindgen(),
        )
    }

    fn get_all(&mut self, index: u32) -> anyhow::Result<Vec<wit::types::EntityId>> {
        Ok(shared_impl::entity::get_all(self.world_mut(), index).into_bindgen())
    }
}
impl wit::component::Component for Bindings {
    fn get_index(&mut self, id: String) -> anyhow::Result<Option<u32>> {
        Ok(shared_impl::entity::get_component_index(&id))
    }

    fn get_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
    ) -> anyhow::Result<Option<wit::component::ComponentTypeResult>> {
        Ok(read_component_from_world(
            self.world(),
            entity.from_bindgen(),
            index,
        ))
    }

    fn add_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::ComponentTypeResult,
    ) -> anyhow::Result<()> {
        Ok(add_component(
            self.world_mut(),
            entity.from_bindgen(),
            index,
            value,
        )?)
    }

    fn add_components(
        &mut self,
        entity: wit::types::EntityId,
        data: ComponentsParam,
    ) -> anyhow::Result<()> {
        Ok(self.world_mut().add_components(
            entity.from_bindgen(),
            convert_components_to_entity_data(data),
        )?)
    }

    fn set_component(
        &mut self,
        entity: wit::types::EntityId,
        index: u32,
        value: wit::component::ComponentTypeResult,
    ) -> anyhow::Result<()> {
        Ok(set_component(
            self.world_mut(),
            entity.from_bindgen(),
            index,
            value,
        )?)
    }

    fn set_components(
        &mut self,
        entity: wit::types::EntityId,
        data: ComponentsParam,
    ) -> anyhow::Result<()> {
        Ok(self.world_mut().set_components(
            entity.from_bindgen(),
            convert_components_to_entity_data(data),
        )?)
    }

    fn has_component(&mut self, entity: wit::types::EntityId, index: u32) -> anyhow::Result<bool> {
        Ok(shared_impl::entity::has_component(
            self.world(),
            entity.from_bindgen(),
            index,
        ))
    }

    fn has_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> anyhow::Result<bool> {
        let mut set = ComponentSet::new();
        for idx in components {
            set.insert_by_index(idx as usize);
        }
        Ok(self.world().has_components(entity.from_bindgen(), &set))
    }

    fn remove_component(&mut self, entity: wit::types::EntityId, index: u32) -> anyhow::Result<()> {
        shared_impl::entity::remove_component(self.world_mut(), entity.from_bindgen(), index)
    }

    fn remove_components(
        &mut self,
        entity: wit::types::EntityId,
        components: Vec<u32>,
    ) -> anyhow::Result<()> {
        let components = with_component_registry(|cr| {
            components
                .into_iter()
                .flat_map(|idx| cr.get_by_index(idx))
                .collect()
        });
        Ok(self
            .world_mut()
            .remove_components(entity.from_bindgen(), components)?)
    }

    fn query(
        &mut self,
        query: wit::component::QueryBuild,
        query_event: wit::component::QueryEvent,
    ) -> anyhow::Result<u64> {
        shared_impl::entity::query(
            &mut self.base.query_states,
            &query.components,
            &query.include,
            &query.exclude,
            &query.changed,
            match query_event {
                wit::component::QueryEvent::Frame => QueryEvent::Frame,
                wit::component::QueryEvent::Spawn => QueryEvent::Spawned,
                wit::component::QueryEvent::Despawn => QueryEvent::Despawned,
            },
        )
    }

    fn query_eval(
        &mut self,
        query_index: u64,
    ) -> anyhow::Result<
        Vec<(
            wit::types::EntityId,
            Vec<wit::component::ComponentTypeResult>,
        )>,
    > {
        let key = slotmap::DefaultKey::from(slotmap::KeyData::from_ffi(query_index));

        let (query, query_state, primitive_components) = self
            .base
            .query_states
            .get(key)
            .context("no query state for key")?;

        let mut query_state = query_state.clone();
        let world = self.world();
        let result = query
            .iter(world, Some(&mut query_state))
            .map(|ea| {
                (
                    ea.id().into_bindgen(),
                    primitive_components
                        .iter()
                        .map(|pc| {
                            read_primitive_component_from_entity_accessor(world, &ea, pc.clone())
                                .unwrap()
                        })
                        .collect(),
                )
            })
            .collect_vec();
        self.base.query_states.get_mut(key).unwrap().1 = query_state;

        Ok(result)
    }
}
impl wit::player::Player for Bindings {
    fn get_raw_input(
        &mut self,
        player: wit::types::EntityId,
    ) -> anyhow::Result<Option<wit::player::RawInput>> {
        Ok(server_impl::player::get_raw_input(self.world(), player.from_bindgen()).into_bindgen())
    }

    fn get_prev_raw_input(
        &mut self,
        player: wit::types::EntityId,
    ) -> anyhow::Result<Option<wit::player::RawInput>> {
        Ok(
            server_impl::player::get_prev_raw_input(self.world(), player.from_bindgen())
                .into_bindgen(),
        )
    }
}
impl wit::physics::Physics for Bindings {
    fn apply_force(
        &mut self,
        entities: Vec<wit::types::EntityId>,
        force: wit::types::Vec3,
    ) -> anyhow::Result<()> {
        let collection = PhysicsObjectCollection::from_entities(
            self.world(),
            &entities.iter().map(|id| id.from_bindgen()).collect_vec(),
        );
        server_impl::physics::apply_force(self.world_mut(), collection, force.from_bindgen())
    }

    fn explode_bomb(
        &mut self,
        position: wit::types::Vec3,
        force: f32,
        radius: f32,
        falloff_radius: Option<f32>,
    ) -> anyhow::Result<()> {
        server_impl::physics::explode_bomb(
            self.world_mut(),
            position.from_bindgen(),
            radius,
            force,
            falloff_radius,
        )
    }

    fn set_gravity(&mut self, gravity: wit::types::Vec3) -> anyhow::Result<()> {
        server_impl::physics::set_gravity(self.world_mut(), gravity.from_bindgen())
    }

    fn unfreeze(&mut self, entity: wit::types::EntityId) -> anyhow::Result<()> {
        server_impl::physics::unfreeze(self.world_mut(), entity.from_bindgen())
    }

    fn freeze(&mut self, entity: wit::types::EntityId) -> anyhow::Result<()> {
        server_impl::physics::freeze(self.world_mut(), entity.from_bindgen())
    }

    fn start_motor(&mut self, entity: wit::types::EntityId, velocity: f32) -> anyhow::Result<()> {
        server_impl::physics::start_motor(self.world_mut(), entity.from_bindgen(), velocity)
    }

    fn stop_motor(&mut self, entity: wit::types::EntityId) -> anyhow::Result<()> {
        server_impl::physics::stop_motor(self.world_mut(), entity.from_bindgen())
    }

    fn raycast_first(
        &mut self,
        origin: wit::types::Vec3,
        direction: wit::types::Vec3,
    ) -> anyhow::Result<Option<(wit::types::EntityId, f32)>> {
        Ok(server_impl::physics::raycast_first(
            self.world(),
            origin.from_bindgen(),
            direction.from_bindgen(),
        )?
        .map(|t| (t.0.into_bindgen(), t.1.into_bindgen())))
    }

    fn raycast(
        &mut self,
        origin: wit::types::Vec3,
        direction: wit::types::Vec3,
    ) -> anyhow::Result<Vec<(wit::types::EntityId, f32)>> {
        Ok(server_impl::physics::raycast(
            self.world(),
            origin.from_bindgen(),
            direction.from_bindgen(),
        )?
        .into_iter()
        .map(|t| (t.0.into_bindgen(), t.1.into_bindgen()))
        .collect())
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
impl wit::event::Event for Bindings {
    fn subscribe(&mut self, name: String) -> anyhow::Result<()> {
        Ok(shared_impl::event::subscribe(
            &mut self.base.subscribed_events,
            &name,
        ))
    }

    fn send(&mut self, name: String, data: ComponentsParam) -> anyhow::Result<()> {
        Ok(shared_impl::event::send(
            self.world_mut(),
            &name,
            convert_components_to_entity_data(data),
        ))
    }
}
impl wit::asset::Asset for Bindings {
    fn url(&mut self, path: String) -> anyhow::Result<Option<String>> {
        let base_url = ServerBaseUrlKey.get(self.world().resource(asset_cache()));
        Ok(Some(AssetUrl::parse(path)?.resolve(&base_url)?.to_string()))
    }
}
impl BindingsBound for Bindings {
    fn base(&self) -> &BindingsBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BindingsBase {
        &mut self.base
    }
}

pub fn initialize(
    world: &mut World,
    messenger: Arc<dyn Fn(&World, EntityId, MessageType, &str) + Send + Sync>,
) -> anyhow::Result<()> {
    crate::shared::initialize(
        world,
        messenger,
        Bindings {
            base: Default::default(),
        },
    )?;

    Ok(())
}
