use std::sync::Arc;

use elements_ecs::{lookup_uid, with_component_registry, QueryEvent, World};
use elements_physics::helpers::PhysicsObjectCollection;
use itertools::Itertools;
use parking_lot::RwLock;
use wit_bindgen_host_wasmtime_rust::Le;

use crate::{
    server::implementation as esei,
    shared::{
        bindings::*,
        conversion::{FromBindgen, IntoBindgen},
        host_guest_state::GetBaseHostGuestState,
        implementation as eshi,
        interface::{self as sif, host},
        BaseWasmContext, WasmContext,
    },
};

pub struct WasmServerContext {
    pub base_context: BaseWasmContext,
    pub elements_bindings: Bindings,
}
impl WasmServerContext {
    pub fn new(
        wasi: wasmtime_wasi::WasiCtx,
        shared_state: Arc<RwLock<dyn GetBaseHostGuestState + Send + Sync>>,
    ) -> Self {
        Self {
            base_context: BaseWasmContext::new(wasi),
            elements_bindings: Bindings::new(shared_state.clone()),
        }
    }

    pub fn link<T>(
        linker: &mut wasmtime::Linker<T>,
        projection: impl Fn(&mut T) -> &mut Self + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()> {
        host::add_to_linker(linker, move |cx| &mut projection(cx).elements_bindings)
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

impl sif::Host for Bindings {
    fn entity_spawn(
        &mut self,
        data: ComponentsParam<'_>,
        persistent: bool,
    ) -> sif::EntityUidResult {
        let id = esei::entity::spawn(
            &mut self.world_mut(),
            convert_components_to_entity_data(data),
        );
        if !persistent {
            self.shared_state
                .write()
                .base_mut()
                .spawned_entities
                .insert(id.clone());
        }
        id.into_bindgen()
    }

    fn entity_spawn_template(
        &mut self,
        object_ref: sif::ObjectRefParam,
        position: sif::Vec3,
        rotation: Option<sif::Quat>,
        scale: Option<sif::Vec3>,
        persistent: bool,
    ) -> sif::EntityUidResult {
        let id = esei::entity::spawn_template(
            &mut self.world_mut(),
            object_ref.id.to_string(),
            position.from_bindgen(),
            rotation.from_bindgen(),
            scale.from_bindgen(),
        )
        .unwrap();
        if !persistent {
            self.shared_state
                .write()
                .base_mut()
                .spawned_entities
                .insert(id.clone());
        }
        id.into_bindgen()
    }

    fn entity_despawn(&mut self, entity: sif::EntityId) -> bool {
        let entity = entity.from_bindgen();
        let despawn = esei::entity::despawn(&mut self.world_mut(), entity);
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
        entity: sif::EntityId,
        animation_controller: sif::AnimationController,
    ) {
        esei::entity::set_animation_controller(
            &mut self.world_mut(),
            entity.from_bindgen(),
            animation_controller.from_bindgen(),
        )
        .unwrap()
    }

    fn entity_set_transform(
        &mut self,
        entity: sif::EntityId,
        transform: sif::Mat4,
        relative: bool,
    ) {
        esei::entity::set_transform(
            &mut self.world_mut(),
            entity.from_bindgen(),
            transform.from_bindgen(),
            relative,
        )
        .unwrap();
    }

    fn entity_get_linear_velocity(&mut self, entity: sif::EntityId) -> Option<sif::Vec3> {
        esei::entity::get_linear_velocity(&mut self.world_mut(), entity.from_bindgen())
            .ok()
            .into_bindgen()
    }

    fn component_get_index(&mut self, id: &str) -> Option<u64> {
        Some(eshi::entity::get_component_index(id)? as u64)
    }

    fn entity_get_component(
        &mut self,
        entity: sif::EntityId,
        index: u64,
    ) -> Option<sif::ComponentTypeResult> {
        read_component_from_world(&self.world(), entity.from_bindgen(), index)
    }

    fn entity_set_component(
        &mut self,
        entity: sif::EntityId,
        index: u64,
        value: sif::ComponentTypeParam,
    ) {
        write_component(&mut self.world_mut(), entity.from_bindgen(), index, value)
    }

    fn entity_set_components(&mut self, entity: sif::EntityId, data: ComponentsParam<'_>) {
        self.world_mut()
            .add_components(
                entity.from_bindgen(),
                convert_components_to_entity_data(data),
            )
            .unwrap()
    }

    fn entity_has_component(&mut self, entity: sif::EntityId, index: u64) -> bool {
        eshi::entity::has_component(&self.world(), entity.from_bindgen(), index as usize)
    }

    fn entity_remove_component(&mut self, entity: sif::EntityId, index: u64) {
        eshi::entity::remove_component(&mut self.world_mut(), entity.from_bindgen(), index as usize)
            .unwrap()
    }

    fn entity_remove_components(&mut self, entity: sif::EntityId, components: &[Le<u64>]) {
        let components = with_component_registry(|cr| {
            components
                .iter()
                .flat_map(|idx| Some(cr.get_by_index(idx.get() as usize)?.clone_boxed()))
                .collect()
        });
        self.world_mut()
            .remove_components(entity.from_bindgen(), components)
            .unwrap()
    }

    fn entity_exists(&mut self, entity: sif::EntityId) -> bool {
        self.world().exists(entity.from_bindgen())
    }

    fn entity_query(&mut self, index: u64) -> Vec<sif::EntityId> {
        eshi::entity::query(&mut self.world_mut(), index).into_bindgen()
    }

    fn entity_query2(&mut self, query: sif::Query, query_event: sif::QueryEvent) -> u64 {
        eshi::entity::query2(
            &mut self.shared_state.write().base_mut().query_states,
            query.components.iter().map(|v| v.get()),
            query.include.iter().map(|v| v.get()),
            query.exclude.iter().map(|v| v.get()),
            query.changed.iter().map(|v| v.get()),
            match query_event {
                sif::QueryEvent::Frame => QueryEvent::Frame,
                sif::QueryEvent::Spawn => QueryEvent::Spawned,
                sif::QueryEvent::Despawn => QueryEvent::Despawned,
            },
        )
        .unwrap()
    }

    fn query_eval(
        &mut self,
        query_index: u64,
    ) -> Vec<(sif::EntityId, Vec<sif::ComponentTypeResult>)> {
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
                                    read_primitive_component_from_entity_accessor(world, &ea, *pc)
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

    fn entity_lookup_uid(&mut self, uid: sif::EntityUidParam<'_>) -> Option<sif::EntityId> {
        lookup_uid(&self.world(), &uid.from_bindgen()).into_bindgen()
    }

    fn entity_in_area(&mut self, position: sif::Vec3, radius: f32) -> Vec<sif::EntityId> {
        eshi::entity::in_area(&mut self.world_mut(), position.from_bindgen(), radius)
            .unwrap()
            .into_bindgen()
    }

    fn physics_apply_force(&mut self, entities: &[Le<sif::EntityId>], force: sif::Vec3) {
        let collection = PhysicsObjectCollection::from_entities(
            &self.world(),
            &entities.iter().map(|id| id.from_bindgen()).collect_vec(),
        );
        esei::physics::apply_force(&mut self.world_mut(), collection, force.from_bindgen()).unwrap()
    }

    fn physics_explode_bomb(
        &mut self,
        position: sif::Vec3,
        force: f32,
        radius: f32,
        falloff_radius: Option<f32>,
    ) {
        esei::physics::explode_bomb(
            &mut self.world_mut(),
            position.from_bindgen(),
            radius,
            force,
            falloff_radius,
        )
        .unwrap();
    }

    fn physics_set_gravity(&mut self, gravity: sif::Vec3) {
        esei::physics::set_gravity(&mut self.world_mut(), gravity.from_bindgen()).unwrap();
    }

    fn physics_unfreeze(&mut self, entity: sif::EntityId) {
        esei::physics::unfreeze(&mut self.world_mut(), entity.from_bindgen()).unwrap();
    }

    fn physics_freeze(&mut self, entity: sif::EntityId) {
        esei::physics::freeze(&mut self.world_mut(), entity.from_bindgen()).unwrap();
    }

    fn physics_start_motor(&mut self, entity: sif::EntityId, velocity: f32) {
        esei::physics::start_motor(&mut self.world_mut(), entity.from_bindgen(), velocity).unwrap();
    }

    fn physics_stop_motor(&mut self, entity: sif::EntityId) {
        esei::physics::stop_motor(&mut self.world_mut(), entity.from_bindgen()).unwrap();
    }

    fn physics_raycast_first(
        &mut self,
        origin: sif::Vec3,
        direction: sif::Vec3,
    ) -> Option<(sif::EntityId, f32)> {
        esei::physics::raycast_first(
            &self.world(),
            origin.from_bindgen(),
            direction.from_bindgen(),
        )
        .unwrap()
        .map(|t| (t.0.into_bindgen(), t.1.into_bindgen()))
    }

    fn physics_raycast(
        &mut self,
        origin: sif::Vec3,
        direction: sif::Vec3,
    ) -> Vec<(sif::EntityId, f32)> {
        esei::physics::raycast(
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
        eshi::event::subscribe(&mut self.shared_state.write().base_mut().event, name)
    }

    fn event_send(&mut self, name: &str, data: ComponentsParam<'_>) {
        eshi::event::send(
            &mut self.shared_state.write().base_mut().event,
            name,
            convert_components_to_entity_data(data),
        )
    }
}
