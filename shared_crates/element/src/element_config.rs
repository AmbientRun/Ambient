use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "guest")]
use ambient_guest_bridge::ecs::UntypedComponent;
use ambient_guest_bridge::ecs::{
    Component, ComponentDesc, ComponentValue, Entity, EntityId, World,
};

use crate::ElementComponent;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub(crate) struct ElementConfig {
    pub part: Option<Box<dyn ElementComponent>>,
    pub components: ElementComponents,
    pub init_components: ElementComponents,
    #[derivative(Debug = "ignore")]
    pub spawner: Arc<dyn Fn(&mut World, Entity) -> EntityId + Sync + Send>,
    #[derivative(Debug = "ignore")]
    pub despawner: Arc<dyn Fn(&mut World, EntityId) + Sync + Send>,
    #[derivative(Debug = "ignore")]
    pub on_spawned: Option<Arc<dyn Fn(&mut World, EntityId, &str) + Sync + Send>>,
    #[derivative(Debug = "ignore")]
    pub on_despawn: Option<Arc<dyn Fn(&mut World, EntityId, &str) + Sync + Send>>,
    pub key: String,
    pub memo_key: Option<String>,
}
impl ElementConfig {
    pub(crate) fn new() -> Self {
        Self {
            part: None,
            components: ElementComponents::new(),
            init_components: ElementComponents::new(),
            spawner: Arc::new(|world, props| world.spawn(props)),
            despawner: Arc::new(|world, entity| {
                world.despawn(entity);
            }),
            on_spawned: None,
            on_despawn: None,
            key: "".to_string(),
            memo_key: None,
        }
    }
    pub(crate) fn get_element_key(&self, short: bool) -> String {
        let name = if let Some(element) = &self.part {
            element.as_ref().element_component_name()
        } else {
            "Entity"
        };
        let name = if short {
            name.split("::").last().unwrap()
        } else {
            name
        };
        if !self.key.is_empty() {
            format!("{}_{}", name, self.key)
        } else if !name.is_empty() {
            name.into()
        } else {
            "unknown".into()
        }
    }
}

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub(crate) struct ElementComponents(
    pub(crate) HashMap<usize, Arc<dyn Fn(&World, &mut Entity) + Sync + Send>>,
);
impl ElementComponents {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn set<T: ComponentValue + Clone + Sync + Send + 'static>(
        &mut self,
        component: Component<T>,
        value: T,
    ) {
        self.set_writer(
            component,
            Arc::new(move |_, ed| ed.set(component, value.clone())),
        );
    }
    pub fn set_writer(
        &mut self,
        component: impl Into<ComponentDesc>,
        writer: Arc<dyn Fn(&World, &mut Entity) + Sync + Send>,
    ) {
        self.0.insert(component.into().index() as _, writer);
    }
    pub fn remove<T: ComponentValue + Clone>(&mut self, component: Component<T>) {
        self.0.remove(&(component.index() as usize));
    }
    pub fn write_to_entity_data(&self, world: &World, entity_data: &mut Entity) {
        for writer in self.0.values() {
            writer(world, entity_data);
        }
    }
}
impl std::fmt::Debug for ElementComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ElementComponents")
    }
}
