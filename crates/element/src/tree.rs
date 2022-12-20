use std::{
    any::TypeId, collections::{HashMap, HashSet}, fmt::Write, sync::Arc
};

use elements_core::hierarchy::{children, parent};
use elements_ecs::{query, Component, EntityData, EntityId, SystemGroup, World};
use itertools::Itertools;
use parking_lot::Mutex;
use tracing::debug_span;

use crate::{
    element_tree, element_unmanaged_children, AnyCloneable, ContextUpdate, DespawnFn, Element, ElementConfig, Hooks, HooksEnvironment, InstanceId
};

#[derive(Debug)]
pub(crate) struct HookContext {
    pub value: Box<dyn AnyCloneable + Sync + Send>,
    pub listeners: HashSet<InstanceId>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct ElementInstance {
    pub id: InstanceId,
    pub super_: Option<InstanceId>,
    pub children: Vec<InstanceId>,
    pub parent: ElementParent,
    pub entity: EntityId,
    pub parent_entity: Option<EntityId>,
    pub config: ElementConfig,
    pub hooks_state: Vec<Box<dyn AnyCloneable + Send>>,
    pub hooks_context_state: HashMap<TypeId, HookContext>,
    pub hooks_context_listening: HashSet<(InstanceId, TypeId)>,

    #[derivative(Debug = "ignore")]
    pub hooks_on_despawn: Vec<DespawnFn>,
}
impl ElementInstance {
    pub fn new(config: ElementConfig, parent_entity: Option<EntityId>, element_parent: ElementParent) -> Self {
        Self {
            id: friendly_id::create(),
            super_: None,
            children: Vec::new(),
            parent: element_parent,
            entity: EntityId::null(),
            parent_entity,
            config,
            hooks_state: Vec::new(),
            hooks_context_state: HashMap::new(),
            hooks_context_listening: HashSet::new(),
            hooks_on_despawn: Vec::new(),
        }
    }

    fn dump(&self, tree: &ElementTree, indent: usize) -> String {
        let mut out = String::new();
        writeln!(&mut out, "{} {}", self.config.get_element_key(true), self.entity).unwrap();
        if let Some(super_) = &self.super_ {
            write!(&mut out, "{:indent$}  - {}", "", tree.dump_instance(super_, indent + 2), indent = indent).unwrap();
        }
        for c in &self.children {
            write!(&mut out, "{:indent$}  o {}", "", tree.dump_instance(c, indent + 2), indent = indent).unwrap();
        }
        out
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ElementParent {
    Super(InstanceId),
    Child(InstanceId, usize),
    None,
}

#[derive(Debug)]
pub struct ElementTree {
    pub(crate) instances: HashMap<InstanceId, ElementInstance>,
    pub(crate) hooks_env: Arc<Mutex<HooksEnvironment>>,
    pub(crate) root: Option<InstanceId>,
}
impl ElementTree {
    pub fn new(world: &mut World, element: Element) -> Self {
        let mut s = Self { instances: HashMap::new(), hooks_env: Arc::new(Mutex::new(HooksEnvironment::new())), root: None };
        let (_, instance) = s.create(world, element, None, ElementParent::None);
        s.update_instance_children(world, &instance);
        s.root = Some(instance);
        s
    }
    pub fn root_entity(&self) -> Option<EntityId> {
        self.root.as_ref().map(|root| {
            let root = self.instances.get(root).unwrap();
            root.entity
        })
    }

    pub fn render_with_component(world: &mut World, id: EntityId, handle: Component<ShareableElementTree>, element: Element) {
        if let Ok(tree) = world.get_ref(id, handle).map(|x| x.clone()) {
            tree.0.lock().migrate_root(world, element);
        } else {
            let tree = ShareableElementTree::new(world, element);
            world.add_component(id, handle, tree).unwrap();
        }
    }
    pub fn render(world: &mut World, id: EntityId, element: Element) {
        Self::render_with_component(world, id, element_tree(), element)
    }
    pub fn systems_for_component(component: Component<ShareableElementTree>) -> SystemGroup {
        SystemGroup::new(
            "ElementTree::systems_for_component",
            vec![
                query((component,)).to_system_with_name("update tree", |q, world, qs, _| {
                    for (_, (tree,)) in q.collect_cloned(world, qs) {
                        tree.0.lock().update(world);
                    }
                }),
                query((component,)).despawned().to_system_with_name("handle despawned", |q, world, qs, _| {
                    for (_, (tree,)) in q.collect_cloned(world, qs) {
                        tree.remove(world);
                    }
                }),
            ],
        )
    }

    fn _super_root(&self, id: &str) -> Option<InstanceId> {
        let instance = self.instances.get(id)?;
        match &instance.parent {
            ElementParent::Super(el) => self._super_root(el),
            ElementParent::Child(_, _) => Some(instance.id.clone()),
            ElementParent::None => Some(instance.id.clone()),
        }
    }

    pub fn n_instances(&self) -> usize {
        self.instances.len()
    }

    pub fn migrate_root(&mut self, world: &mut World, element: Element) {
        if let Some((_, new_root)) = self.migrate(world, self.root.clone(), None, ElementParent::None, Some(element)) {
            self.root = Some(new_root);
        }
    }

    pub fn remove_root(&mut self, world: &mut World) {
        if let Some(root) = self.root.clone() {
            self.remove(world, &root);
        }
        self.root = None;
    }

    fn create(
        &mut self,
        world: &mut World,
        element: Element,
        parent_entity: Option<EntityId>,
        element_parent: ElementParent,
    ) -> (EntityId, InstanceId) {
        let instance = ElementInstance::new(element.config, parent_entity, element_parent);
        let id = instance.id.clone();
        self.instances.insert(instance.id.clone(), instance);
        let entity = self.render_instance(world, &id, true);
        let children = element
            .children
            .into_iter()
            .enumerate()
            .map(|(i, child)| {
                let child_id = self.create(world, child, Some(entity), ElementParent::Child(id.clone(), i)).1;
                self.update_instance_children(world, &child_id);
                child_id
            })
            .collect_vec();
        self.instances.get_mut(&id).unwrap().children = children;
        (entity, id)
    }

    fn render_instance(&mut self, world: &mut World, instance_id: &str, creating: bool) -> EntityId {
        let instance = self.instances.get(instance_id).unwrap();
        let key = instance.config.get_element_key(true);

        profiling::scope!("render_instance", key.clone());
        let part = instance.config.part.clone();

        let entity = if let Some(part) = part {
            // Clear frame listeners as they are rebuilt during render
            self.hooks_env.lock().frame_listeners.remove(instance_id);
            let _span = debug_span!("render_instance with part", key).entered();
            let (on_spawn, new_super, super_, parent_entity) = {
                let mut hooks = Hooks {
                    environment: self.hooks_env.clone(),
                    tree: self,
                    element: instance_id.to_string(),
                    state_index: 0,
                    on_spawn: if creating { Some(Vec::new()) } else { None },
                };

                // eprintln!("Calling part::render on: {key}");
                let new_super = part.render(world, &mut hooks);
                let on_spawn = std::mem::take(&mut hooks.on_spawn);

                drop(hooks);
                let instance = self.instances.get_mut(instance_id).unwrap();
                (on_spawn, new_super, instance.super_.clone(), instance.parent_entity)
            };

            let (ent, new_super) =
                self.migrate(world, super_, parent_entity, ElementParent::Super(instance_id.to_string()), Some(new_super)).unwrap();

            let instance = self.instances.get_mut(instance_id).unwrap();
            instance.super_ = Some(new_super);
            if creating {
                instance.hooks_on_despawn = on_spawn.into_iter().flatten().map(|f| f(world)).collect_vec();
            }
            ent
        } else {
            let instance = self.instances.get(instance_id).unwrap();
            if instance.entity.is_null() {
                let mut entity_data = EntityData::new().set_default(crate::element());
                if let Some(parent_entity) = instance.parent_entity {
                    entity_data = entity_data.set(parent(), parent_entity);
                }
                (instance.config.spawner)(world, entity_data)
            } else {
                instance.entity
            }
        };

        let mut components = EntityData::new();
        let spawn = {
            let instance = self.instances.get_mut(instance_id).unwrap();
            let spawn = instance.entity != entity;
            if spawn {
                instance.config.init_components.write_to_entity_data(world, &mut components);
                let name = world.get_ref(entity, crate::element()).unwrap().clone();
                world.set(entity, crate::element(), format!("{}({})/{}", instance.config.get_element_key(true), entity, name)).unwrap();
            }
            instance.entity = entity;
            spawn
        };
        self.gather_parent_components(world, instance_id, &mut components);
        let instance = self.instances.get(instance_id).unwrap();
        world.add_components(instance.entity, components).unwrap();
        instance.config.event_listeners.add_to_entity(world, instance.entity);
        if spawn {
            if let Some(on_spawned) = &instance.config.on_spawned {
                on_spawned(world, entity);
            }
        }
        instance.entity
    }
    fn rerender_instance(&mut self, world: &mut World, instance_id: &str) {
        let old_entity = if let Some(instance) = self.instances.get(instance_id) {
            instance.entity
        } else {
            return;
        };

        self.render_instance(world, instance_id, false);
        let instance = self.instances.get_mut(instance_id).unwrap();
        if instance.entity != old_entity {
            if let Some(parent) = instance.parent_entity {
                let children = world.get_mut(parent, children()).unwrap();
                for c in children.iter_mut() {
                    if *c == old_entity {
                        *c = instance.entity;
                    }
                }
            }
        }
    }
    fn migrate(
        &mut self,
        world: &mut World,
        old_node_id: Option<InstanceId>,
        node_parent: Option<EntityId>,
        element_parent: ElementParent,
        new_node: Option<Element>,
    ) -> Option<(EntityId, InstanceId)> {
        let res = match (old_node_id, new_node) {
            (Some(old_node_id), Some(new_node)) => {
                let (old_node_config_memo_key, old_node_entity, old_key) = {
                    let old_node = self.instances.get(&old_node_id).unwrap();
                    (old_node.config.memo_key.clone(), old_node.entity, old_node.config.get_element_key(false))
                };
                if new_node.config.memo_key.is_some() && old_node_config_memo_key == new_node.config.memo_key {
                    Some((old_node_entity, old_node_id.clone()))
                } else {
                    let new_key = new_node.config.get_element_key(false);
                    let res = if old_key == new_key {
                        self.migrate_instance(world, &old_node_id, node_parent, new_node);
                        (old_node_entity, old_node_id.clone())
                    } else {
                        self.remove(world, &old_node_id);
                        self.create(world, new_node, node_parent, element_parent)
                    };
                    Some(res)
                }
            }
            (None, Some(new_node)) => Some(self.create(world, new_node, node_parent, element_parent)),
            (Some(old_node_id), None) => {
                self.remove(world, &old_node_id);
                None
            }
            (None, None) => None,
        };
        if let Some((_, id)) = &res {
            self.update_instance_children(world, id);
        }
        res
    }
    fn migrate_instance(&mut self, world: &mut World, instance_id: &str, node_parent: Option<EntityId>, new_node: Element) {
        {
            let instance = self.instances.get_mut(instance_id).unwrap();
            instance.config.event_listeners.remove_from_entity(world, instance.entity);
            instance.config = new_node.config;
            instance.parent_entity = node_parent;
        }
        let entity = self.render_instance(world, instance_id, false);

        // Migrate children
        let mut new_children = Vec::new();
        let instance_children = self.instances.get(instance_id).unwrap().children.clone();
        for i in 0..(new_node.children.len().max(instance_children.len())) {
            let old_child = instance_children.get(i).cloned();
            if i < new_node.children.len() {
                let new_child = new_node.children.get(i).cloned();
                let (_, ret_node) =
                    self.migrate(world, old_child, Some(entity), ElementParent::Child(instance_id.to_string(), i), new_child).unwrap();
                new_children.push(ret_node);
            } else {
                self.remove(world, &old_child.unwrap());
            }
        }
        self.instances.get_mut(instance_id).unwrap().children = new_children;
    }
    fn remove(&mut self, world: &mut World, instance_id: &str) {
        let mut instance = self.instances.remove(instance_id).unwrap();

        if let Some(on_despawn) = &instance.config.on_despawn {
            on_despawn(world, instance.entity);
        }
        for on_despawn in std::mem::take(&mut instance.hooks_on_despawn) {
            on_despawn(world);
        }
        if instance.config.part.is_none() {
            (instance.config.despawner)(world, instance.entity);
        }
        instance.entity = EntityId::null();
        instance.hooks_state = Vec::new();
        self.hooks_env.lock().on_element_removed(instance_id);

        for (id, type_id) in &instance.hooks_context_listening {
            if let Some(instance) = self.instances.get_mut(id) {
                if let Some(ctx) = instance.hooks_context_state.get_mut(type_id) {
                    ctx.listeners.remove(instance_id);
                }
            }
        }

        if let Some(super_) = &instance.super_ {
            self.remove(world, super_);
        }
        for child in &instance.children {
            self.remove(world, child);
        }
    }

    #[profiling::function]
    pub fn update(&mut self, world: &mut World) {
        let frame_listeners = self.hooks_env.lock().frame_listeners.clone();
        for listeners in frame_listeners.values() {
            profiling::scope!("frame_listeners");
            for listener in listeners {
                listener.0(world);
            }
        }
        let state_updates = std::mem::take(&mut self.hooks_env.lock().set_states);
        let context_updates = std::mem::take(&mut self.hooks_env.lock().set_contexts);
        let mut to_update = HashSet::new();
        for (instance_id, index, value) in state_updates.into_iter() {
            profiling::scope!("state_updates");
            if let Some(instance) = self.instances.get_mut(&instance_id) {
                let key = &instance.config.get_element_key(true);
                tracing::debug!(key, "updated state");
                instance.hooks_state[index] = value;
                to_update.insert(instance_id);
            }
        }
        for ContextUpdate { instance_id, type_id, name, value } in context_updates.into_iter() {
            profiling::scope!("state_updates");

            if let Some(instance) = self.instances.get_mut(&instance_id) {
                let key = &instance.config.get_element_key(true);
                tracing::debug!(key, "Subscribed context {name:?} was updated");
                let entry = instance.hooks_context_state.get_mut(&type_id).unwrap();
                entry.value = value;
                to_update.extend(entry.listeners.iter().cloned());
            }
        }
        for instance_id in to_update.into_iter() {
            profiling::scope!("rerender_instance", &instance_id);
            self.rerender_instance(world, &instance_id);
        }
    }
    // TODO: Maybe optimize when this is called. It's kind of just called everywhere "just in case" now
    fn update_instance_children(&mut self, world: &mut World, id: &str) {
        let instance = self.instances.get(id).unwrap();
        if !world.has_component(instance.entity, element_unmanaged_children()) {
            let mut all_children = Vec::new();
            self.get_full_instance_children(id, &mut all_children);
            world
                .add_component(
                    instance.entity,
                    children(),
                    all_children.iter().map(|c| self.instances.get(c).unwrap().entity).collect_vec(),
                )
                .unwrap();
        }
    }
    fn gather_parent_components(&self, world: &World, instance_id: &str, components: &mut EntityData) {
        let parent = {
            let instance = self.instances.get(instance_id).unwrap();
            instance.config.components.write_to_entity_data(world, components);
            instance.parent.clone()
        };
        if let ElementParent::Super(super_) = &parent {
            self.gather_parent_components(world, super_, components);
        }
    }
    fn get_full_instance_children(&self, id: &str, children: &mut Vec<String>) {
        let instance = self.instances.get(id).unwrap();
        if let Some(super_) = &instance.super_ {
            self.get_full_instance_children(super_, children);
        }
        children.extend(instance.children.clone());
    }
    pub(crate) fn get_context_provider(&self, instance_id: &str, context_type_id: TypeId) -> Option<String> {
        let instance = self.instances.get(instance_id).unwrap();
        if instance.hooks_context_state.contains_key(&context_type_id) {
            Some(instance_id.to_string())
        } else {
            match &instance.parent {
                ElementParent::Super(super_) => self.get_context_provider(super_, context_type_id),
                ElementParent::Child(id, _) => self.get_context_provider(id, context_type_id),
                ElementParent::None => None,
            }
        }
    }
    fn dump_instance(&self, instance_id: &str, indent: usize) -> String {
        let instance = self.instances.get(instance_id).unwrap();
        instance.dump(self, indent)
    }
    pub fn dump(&self, indent: usize) -> String {
        if let Some(root) = self.root.clone() {
            self.dump_instance(&root, indent)
        } else {
            "No root".to_string()
        }
    }
    pub fn dump_to_tmp_file(&self) {
        std::fs::write("tmp/elements.txt", self.dump(0)).expect("Unable to write file");
        println!("Wrote elements to tmp/elements.txt");
    }
}
impl std::fmt::Display for ElementTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dump(0))
    }
}

#[derive(Debug, Clone)]
pub struct ShareableElementTree(pub Arc<Mutex<ElementTree>>);
impl ShareableElementTree {
    pub fn new(world: &mut World, element: Element) -> Self {
        Self(Arc::new(Mutex::new(ElementTree::new(world, element))))
    }
    pub fn remove(&self, world: &mut World) {
        self.0.lock().remove_root(world);
    }
}
impl std::fmt::Display for ShareableElementTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.lock().dump(0))
    }
}
impl From<ElementTree> for ShareableElementTree {
    fn from(tree: ElementTree) -> Self {
        Self(Arc::new(Mutex::new(tree)))
    }
}
