use std::{collections::HashMap, sync::Arc};

use elements_ecs::{Component, ComponentValue, EntityData, EntityId, IComponent, World};
use elements_std::events::EventDispatcher;

use crate::ElementComponent;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub(crate) struct ElementConfig {
    pub part: Option<Box<dyn ElementComponent>>,
    pub components: ElementComponents,
    pub init_components: ElementComponents,
    pub event_listeners: ElementEventHandlers,
    #[derivative(Debug = "ignore")]
    pub spawner: Arc<dyn Fn(&mut World, EntityData) -> EntityId + Sync + Send>,
    #[derivative(Debug = "ignore")]
    pub despawner: Arc<dyn Fn(&mut World, EntityId) + Sync + Send>,
    #[derivative(Debug = "ignore")]
    pub on_spawned: Option<Arc<dyn Fn(&mut World, EntityId) + Sync + Send>>,
    #[derivative(Debug = "ignore")]
    pub on_despawn: Option<Arc<dyn Fn(&mut World, EntityId) + Sync + Send>>,
    pub key: String,
    pub memo_key: Option<String>,
}
impl ElementConfig {
    pub(crate) fn new() -> Self {
        Self {
            part: None,
            components: ElementComponents::new(),
            init_components: ElementComponents::new(),
            event_listeners: ElementEventHandlers::new(),
            spawner: Arc::new(|world, props| props.spawn(world)),
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
        let name = if let Some(element) = &self.part { element.as_ref().part_name() } else { "Entity" };
        let name = if short { name.split("::").last().unwrap() } else { name };
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
pub(crate) struct ElementComponents(pub(crate) HashMap<usize, Arc<dyn Fn(&World, &mut EntityData) + Sync + Send>>);
impl ElementComponents {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn set<T: ComponentValue + Clone>(&mut self, component: Component<T>, value: T) {
        self.set_writer(&component, Arc::new(move |_, ed| ed.set_self(component, value.clone())));
    }
    pub fn set_writer(&mut self, component: &dyn IComponent, writer: Arc<dyn Fn(&World, &mut EntityData) + Sync + Send>) {
        self.0.insert(component.get_index(), writer);
    }
    pub fn remove<T: ComponentValue + Clone>(&mut self, component: Component<T>) {
        self.0.remove(&component.get_index());
    }
    pub fn write_to_entity_data(&self, world: &World, entity_data: &mut EntityData) {
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

#[derive(Clone)]
pub(crate) struct ElementEventHandler {
    add: Arc<dyn Fn(&mut World, EntityId) + Sync + Send>,
    remove: Arc<dyn Fn(&mut World, EntityId) + Sync + Send>,
}

#[derive(Clone)]
pub(crate) struct ElementEventHandlers(HashMap<usize, Vec<ElementEventHandler>>);
impl ElementEventHandlers {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn set<T: 'static + Sync + Send + ?Sized>(&mut self, component: Component<EventDispatcher<T>>, listener: Arc<T>) {
        let entry = self.0.entry(component.get_index()).or_default();
        entry.push(ElementEventHandler {
            add: Arc::new(closure!(clone listener, |world, entity| {
                if let Ok(event_dispatcher) = world.get_mut(entity, component) {
                    event_dispatcher.add(listener.clone());
                } else {
                    world.add_component(entity, component, EventDispatcher::<T>::new_with(listener.clone())).unwrap();
                }
            })),
            remove: Arc::new(move |world, entity| {
                let event_dispatcher = world.get_mut(entity, component).unwrap();
                event_dispatcher.remove(listener.clone());
            }),
        });
    }
    pub fn add_to_entity(&self, world: &mut World, entity: EntityId) {
        for handlers in self.0.values() {
            for handler in handlers {
                (handler.add)(world, entity);
            }
        }
    }
    pub fn remove_from_entity(&self, world: &mut World, entity: EntityId) {
        for handlers in self.0.values() {
            for handler in handlers {
                (handler.remove)(world, entity);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl std::fmt::Debug for ElementEventHandlers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ElementEventHandlers")
    }
}
