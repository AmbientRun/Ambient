use std::sync::Arc;

use ambient_ecs::{with_component_registry, ComponentSet, QueryEvent, World};
use ambient_physics::helpers::PhysicsObjectCollection;
use itertools::Itertools;
use parking_lot::RwLock;
use wit_bindgen_host_wasmtime_rust::Le;

use crate::{
    server::implementation as server_impl,
    shared::{
        bindings::*,
        conversion::{FromBindgen, IntoBindgen},
        host_guest_state::GetBaseHostGuestState,
        implementation as shared_impl,
        interface::host,
        BaseWasmContext, WasmContext,
    },
};
use ambient_core::asset_cache;
use ambient_std::{
    asset_cache::SyncAssetKeyExt,
    asset_url::{AssetUrl, ServerBaseUrlKey},
};

pub struct WasmServerContext {
    pub base_context: BaseWasmContext,
    pub ambient_bindings: Bindings,
}
impl WasmServerContext {
    pub fn new(
        wasi: wasmtime_wasi::WasiCtx,
        shared_state: Arc<RwLock<dyn GetBaseHostGuestState + Send + Sync>>,
    ) -> Self {
        Self {
            base_context: BaseWasmContext::new(wasi),
            ambient_bindings: Bindings::new(shared_state.clone()),
        }
    }

    pub fn link<T>(
        linker: &mut wasmtime::Linker<T>,
        projection: impl Fn(&mut T) -> &mut Self + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()> {
        host::add_to_linker(linker, move |cx| &mut projection(cx).ambient_bindings)
    }
}
impl WasmContext<Bindings> for WasmServerContext {
    fn base_wasm_context_mut(&mut self) -> &mut BaseWasmContext {
        &mut self.base_context
    }
}

pub struct Bindings {
    shared_state: Arc<RwLock<dyn GetBaseHostGuestState + Send + Sync>>,
}
impl Bindings {
    fn new(shared_state: Arc<RwLock<dyn GetBaseHostGuestState + Send + Sync>>) -> Self {
        Self { shared_state }
    }
    fn world(&self) -> parking_lot::MappedRwLockReadGuard<World> {
        parking_lot::RwLockReadGuard::map(self.shared_state.read(), |s| s.base().world())
    }
    fn world_mut(&mut self) -> parking_lot::MappedRwLockWriteGuard<World> {
        parking_lot::RwLockWriteGuard::map(self.shared_state.write(), |s| s.base_mut().world_mut())
    }
}

impl host::Host for Bindings {
    fn entity_spawn(&mut self, data: ComponentsParam<'_>) -> host::EntityId {
        let id = shared_impl::entity::spawn(
            &mut self.world_mut(),
            convert_components_to_entity_data(data),
        );
        self.shared_state
            .write()
            .base_mut()
            .spawned_entities
            .insert(id);
        id.into_bindgen()
    }

    fn entity_despawn(&mut self, entity: host::EntityId) -> bool {
        let entity = entity.from_bindgen();
        let despawn = shared_impl::entity::despawn(&mut self.world_mut(), entity);
        if let Some(uid) = despawn {
            self.shared_state
                .write()
                .base_mut()
                .spawned_entities
                .remove(&uid);
            true
        } else {
            false
        }
    }

    fn entity_set_animation_controller(
        &mut self,
        entity: host::EntityId,
        animation_controller: host::AnimationController,
    ) {
        shared_impl::entity::set_animation_controller(
            &mut self.world_mut(),
            entity.from_bindgen(),
            animation_controller.from_bindgen(),
        )
        .unwrap()
    }

    fn component_get_index(&mut self, id: &str) -> Option<u32> {
        shared_impl::entity::get_component_index(id)
    }

    fn entity_get_component(
        &mut self,
        entity: host::EntityId,
        index: u32,
    ) -> Option<host::ComponentTypeResult> {
        read_component_from_world(&self.world(), entity.from_bindgen(), index)
    }

    fn entity_add_component(
        &mut self,
        entity: host::EntityId,
        index: u32,
        value: host::ComponentTypeParam,
    ) {
        add_component(&mut self.world_mut(), entity.from_bindgen(), index, value).unwrap()
    }

    fn entity_add_components(&mut self, entity: host::EntityId, data: ComponentsParam<'_>) {
        self.world_mut()
            .add_components(
                entity.from_bindgen(),
                convert_components_to_entity_data(data),
            )
            .unwrap()
    }

    fn entity_set_component(
        &mut self,
        entity: host::EntityId,
        index: u32,
        value: host::ComponentTypeParam,
    ) {
        set_component(&mut self.world_mut(), entity.from_bindgen(), index, value).unwrap()
    }

    fn entity_set_components(&mut self, entity: host::EntityId, data: ComponentsParam<'_>) {
        self.world_mut()
            .set_components(
                entity.from_bindgen(),
                convert_components_to_entity_data(data),
            )
            .unwrap()
    }

    fn entity_has_component(&mut self, entity: host::EntityId, index: u32) -> bool {
        shared_impl::entity::has_component(&self.world(), entity.from_bindgen(), index)
    }

    fn entity_has_components(&mut self, entity: host::EntityId, components: &[Le<u32>]) -> bool {
        let mut set = ComponentSet::new();
        for idx in components {
            set.insert_by_index(idx.get() as usize);
        }
        self.world().has_components(entity.from_bindgen(), &set)
    }

    fn entity_remove_component(&mut self, entity: host::EntityId, index: u32) {
        shared_impl::entity::remove_component(&mut self.world_mut(), entity.from_bindgen(), index)
            .unwrap()
    }

    fn entity_remove_components(&mut self, entity: host::EntityId, components: &[Le<u32>]) {
        let components = with_component_registry(|cr| {
            components
                .iter()
                .flat_map(|idx| cr.get_by_index(idx.get()))
                .collect()
        });
        self.world_mut()
            .remove_components(entity.from_bindgen(), components)
            .unwrap()
    }

    fn entity_exists(&mut self, entity: host::EntityId) -> bool {
        self.world().exists(entity.from_bindgen())
    }

    fn entity_query(&mut self, index: u32) -> Vec<host::EntityId> {
        shared_impl::entity::query(&mut self.world_mut(), index).into_bindgen()
    }

    fn entity_query2(&mut self, query: host::Query, query_event: host::QueryEvent) -> u64 {
        shared_impl::entity::query2(
            &mut self.shared_state.write().base_mut().query_states,
            query.components.iter().map(|v| v.get()),
            query.include.iter().map(|v| v.get()),
            query.exclude.iter().map(|v| v.get()),
            query.changed.iter().map(|v| v.get()),
            match query_event {
                host::QueryEvent::Frame => QueryEvent::Frame,
                host::QueryEvent::Spawn => QueryEvent::Spawned,
                host::QueryEvent::Despawn => QueryEvent::Despawned,
            },
        )
        .unwrap()
    }

    fn query_eval(
        &mut self,
        query_index: u64,
    ) -> Vec<(host::EntityId, Vec<host::ComponentTypeResult>)> {
        let key = slotmap::DefaultKey::from(slotmap::KeyData::from_ffi(query_index));
        let shared_state = self.shared_state.clone();
        let (result, query_state) = {
            let lock = shared_state.read();
            let base = lock.base();
            let (query, query_state, primitive_components) =
                base.query_states.get(key).expect("no query state for key");

            let mut query_state = query_state.clone();

            let world = base.world();
            (
                query
                    .iter(world, Some(&mut query_state))
                    .map(|ea| {
                        (
                            ea.id().into_bindgen(),
                            primitive_components
                                .iter()
                                .map(|pc| {
                                    read_primitive_component_from_entity_accessor(
                                        world,
                                        &ea,
                                        pc.clone(),
                                    )
                                    .unwrap()
                                })
                                .collect(),
                        )
                    })
                    .collect_vec(),
                query_state,
            )
        };
        shared_state
            .write()
            .base_mut()
            .query_states
            .get_mut(key)
            .unwrap()
            .1 = query_state;

        result
    }

    fn entity_resources(&mut self) -> host::EntityId {
        shared_impl::entity::resources(&self.world()).into_bindgen()
    }

    fn entity_in_area(&mut self, position: host::Vec3, radius: f32) -> Vec<host::EntityId> {
        shared_impl::entity::in_area(&mut self.world_mut(), position.from_bindgen(), radius)
            .unwrap()
            .into_bindgen()
    }

    fn player_get_raw_input(&mut self, player: host::EntityId) -> Option<host::PlayerRawInput> {
        server_impl::player::get_raw_input(&self.world(), player.from_bindgen()).into_bindgen()
    }

    fn player_get_prev_raw_input(
        &mut self,
        player: host::EntityId,
    ) -> Option<host::PlayerRawInput> {
        server_impl::player::get_prev_raw_input(&self.world(), player.from_bindgen()).into_bindgen()
    }

    fn physics_apply_force(&mut self, entities: &[Le<host::EntityId>], force: host::Vec3) {
        let collection = PhysicsObjectCollection::from_entities(
            &self.world(),
            &entities.iter().map(|id| id.from_bindgen()).collect_vec(),
        );
        server_impl::physics::apply_force(&mut self.world_mut(), collection, force.from_bindgen())
            .unwrap()
    }

    fn physics_explode_bomb(
        &mut self,
        position: host::Vec3,
        force: f32,
        radius: f32,
        falloff_radius: Option<f32>,
    ) {
        server_impl::physics::explode_bomb(
            &mut self.world_mut(),
            position.from_bindgen(),
            radius,
            force,
            falloff_radius,
        )
        .unwrap();
    }

    fn physics_set_gravity(&mut self, gravity: host::Vec3) {
        server_impl::physics::set_gravity(&mut self.world_mut(), gravity.from_bindgen()).unwrap();
    }

    fn physics_unfreeze(&mut self, entity: host::EntityId) {
        server_impl::physics::unfreeze(&mut self.world_mut(), entity.from_bindgen()).unwrap();
    }

    fn physics_freeze(&mut self, entity: host::EntityId) {
        server_impl::physics::freeze(&mut self.world_mut(), entity.from_bindgen()).unwrap();
    }

    fn physics_start_motor(&mut self, entity: host::EntityId, velocity: f32) {
        server_impl::physics::start_motor(&mut self.world_mut(), entity.from_bindgen(), velocity)
            .unwrap();
    }

    fn physics_stop_motor(&mut self, entity: host::EntityId) {
        server_impl::physics::stop_motor(&mut self.world_mut(), entity.from_bindgen()).unwrap();
    }

    fn physics_raycast_first(
        &mut self,
        origin: host::Vec3,
        direction: host::Vec3,
    ) -> Option<(host::EntityId, f32)> {
        server_impl::physics::raycast_first(
            &self.world(),
            origin.from_bindgen(),
            direction.from_bindgen(),
        )
        .unwrap()
        .map(|t| (t.0.into_bindgen(), t.1.into_bindgen()))
    }

    fn physics_raycast(
        &mut self,
        origin: host::Vec3,
        direction: host::Vec3,
    ) -> Vec<(host::EntityId, f32)> {
        server_impl::physics::raycast(
            &self.world(),
            origin.from_bindgen(),
            direction.from_bindgen(),
        )
        .unwrap()
        .into_iter()
        .map(|t| (t.0.into_bindgen(), t.1.into_bindgen()))
        .collect()
    }

    fn event_subscribe(&mut self, name: &str) {
        shared_impl::event::subscribe(&mut self.shared_state.write().base_mut().event, name)
    }

    fn event_send(&mut self, name: &str, data: ComponentsParam<'_>) {
        shared_impl::event::send(
            &mut self.shared_state.write().base_mut().event,
            name,
            convert_components_to_entity_data(data),
        )
    }

    fn asset_url(&mut self, path: &str) -> Option<String> {
        let base_url = ServerBaseUrlKey.get(self.world().resource(asset_cache()));
        AssetUrl::parse(path)
            .ok()?
            .resolve(&base_url)
            .ok()
            .map(|x| x.to_string())
    }
}
