use glam::Mat4;

use crate::{
    ecs::{ComponentValue, Result},
    global::{EntityId, Vec3},
    internal::{
        component::{Entity, UntypedComponent},
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
    prelude::ECSError,
};

pub struct HostWorld;
impl HostWorld {
    pub fn spawn(&mut self, components: Entity) -> EntityId {
        wit::entity::spawn(&components.into_bindgen()).from_bindgen()
    }

    pub fn despawn(&mut self, entity: EntityId) -> Option<Entity> {
        wit::entity::despawn(entity.into_bindgen()).from_bindgen()
    }

    pub fn get_transforms_relative_to(&self, list: &[EntityId], origin: EntityId) -> Vec<Mat4> {
        wit::entity::get_transforms_relative_to(
            &list.iter().map(|x| x.into_bindgen()).collect::<Vec<_>>(),
            origin.into_bindgen(),
        )
        .from_bindgen()
    }

    pub fn exists(&self, entity: EntityId) -> bool {
        wit::entity::exists(entity.into_bindgen())
    }

    pub fn get_all_untyped(&self, component: &dyn UntypedComponent) -> Vec<EntityId> {
        wit::entity::get_all(component.index()).from_bindgen()
    }

    pub fn in_area(&self, position: Vec3, radius: f32) -> Vec<EntityId> {
        wit::entity::in_area(position.into_bindgen(), radius).from_bindgen()
    }

    pub fn get_component_untyped(
        &self,
        entity: EntityId,
        component: &dyn UntypedComponent,
    ) -> Result<ComponentValue> {
        // TODO: Need to actually get a meaningful error
        wit::component::get_component(entity.into_bindgen(), component.index())
            .from_bindgen()
            .ok_or(ECSError::NoSuchEntity)
    }

    pub fn get_components(&self, entity: EntityId, components: &[&dyn UntypedComponent]) -> Entity {
        let components: Vec<_> = components.iter().map(|c| c.index()).collect();
        wit::component::get_components(entity.into_bindgen(), &components).from_bindgen()
    }

    pub fn get_all_components(&self, entity: EntityId) -> Entity {
        wit::component::get_all_components(entity.into_bindgen()).from_bindgen()
    }

    pub fn add_component_untyped(
        &mut self,
        entity: EntityId,
        component: &dyn UntypedComponent,
        value: ComponentValue,
    ) -> Result<()> {
        wit::component::add_component(
            entity.into_bindgen(),
            component.index(),
            &value.into_bindgen(),
        );
        Ok(())
    }

    pub fn add_components(&mut self, entity: EntityId, components: Entity) -> Result<()> {
        wit::component::add_components(entity.into_bindgen(), &components.into_bindgen());
        Ok(())
    }

    pub fn set_component_untyped(
        &mut self,
        entity: EntityId,
        component: &dyn UntypedComponent,
        value: ComponentValue,
    ) -> Result<()> {
        wit::component::set_component(
            entity.into_bindgen(),
            component.index(),
            &value.into_bindgen(),
        );
        Ok(())
    }

    pub fn set_components(&mut self, entity: EntityId, components: Entity) {
        wit::component::set_components(entity.into_bindgen(), &components.into_bindgen())
    }

    pub fn has_component_untyped(
        &self,
        entity: EntityId,
        component: &dyn UntypedComponent,
    ) -> bool {
        wit::component::has_component(entity.into_bindgen(), component.index())
    }

    pub fn has_components(&self, entity: EntityId, components: &[&dyn UntypedComponent]) -> bool {
        let components: Vec<_> = components.iter().map(|c| c.index()).collect();
        wit::component::has_components(entity.into_bindgen(), &components)
    }

    pub fn remove_component_untyped(&mut self, entity: EntityId, component: &dyn UntypedComponent) {
        wit::component::remove_component(entity.into_bindgen(), component.index())
    }

    pub fn remove_components(&mut self, entity: EntityId, components: &[&dyn UntypedComponent]) {
        let components: Vec<_> = components.iter().map(|c| c.index()).collect();
        wit::component::remove_components(entity.into_bindgen(), &components)
    }

    pub fn resources(&self) -> EntityId {
        EntityId::resources()
    }

    pub fn synchronized_resources(&self) -> EntityId {
        wit::entity::synchronized_resources().from_bindgen()
    }

    pub fn persisted_resources(&self) -> EntityId {
        wit::entity::persisted_resources().from_bindgen()
    }
}
