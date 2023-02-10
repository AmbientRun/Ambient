use std::collections::{hash_map::Entry, BTreeMap, HashMap};

use elements_std::events::EventDispatcher;
use once_cell::sync::Lazy;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::*;
use crate::ComponentVTable;

static COMPONENT_REGISTRY: Lazy<RwLock<ComponentRegistry>> = Lazy::new(|| RwLock::new(ComponentRegistry::default()));
static COMPONENT_ATTRIBUTES: RwLock<BTreeMap<u32, AttributeStore>> = RwLock::new(BTreeMap::new());

pub(crate) fn get_external_attributes(index: u32) -> AttributeStoreGuard {
    let guard = COMPONENT_ATTRIBUTES.read();

    RwLockReadGuard::map(guard, |val| val.get(&index).expect("No external attributes"))
}

pub(crate) fn get_external_attributes_init(index: u32) -> AttributeStoreGuardMut {
    let guard = COMPONENT_ATTRIBUTES.write();

    RwLockWriteGuard::map(guard, |val| val.entry(index).or_default())
}

pub fn with_component_registry<R>(f: impl FnOnce(&ComponentRegistry) -> R + Sync + Send) -> R {
    let lock = COMPONENT_REGISTRY.read();
    f(&lock)
}

pub(crate) struct RegistryComponent {
    pub(crate) desc: ComponentDesc,
    pub(crate) primitive_component: Option<PrimitiveComponent>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExternalComponentDesc {
    pub path: String,
    pub ty: PrimitiveComponentType,
    pub attributes: ExternalComponentAttributes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExternalComponentAttributes {
    pub name: Option<String>,
    pub description: Option<String>,
    pub debuggable: bool,
    pub networked: bool,
    pub store: bool,
}
impl ExternalComponentAttributes {
    pub fn from_existing_component(desc: ComponentDesc) -> Self {
        Self {
            name: desc.attribute::<Name>().map(|n| n.0.clone()),
            description: desc.attribute::<Description>().map(|n| n.0.clone()),
            debuggable: desc.has_attribute::<Debuggable>(),
            networked: desc.has_attribute::<Networked>(),
            store: desc.has_attribute::<Store>(),
        }
    }
}

#[derive(Default)]
pub struct ComponentRegistry {
    pub(crate) components: Vec<RegistryComponent>,
    pub component_paths: HashMap<String, u32>,
    pub next_index: u32,

    /// Handlers are called with a write-lock on ComponentRegistry, which will result in deadlock if your operation
    /// requires a read-lock on ComponentRegistry. Consider deferring your operation to a later time.
    pub on_external_components_change: EventDispatcher<dyn Fn() + Sync + Send>,
}
impl ComponentRegistry {
    pub fn get() -> RwLockReadGuard<'static, Self> {
        COMPONENT_REGISTRY.read()
    }
    pub fn get_mut() -> RwLockWriteGuard<'static, Self> {
        COMPONENT_REGISTRY.write()
    }

    pub fn add_external(&mut self, components: Vec<ExternalComponentDesc>) {
        for desc in components {
            desc.ty.register(self, &desc.path, desc.attributes);
        }

        for handler in self.on_external_components_change.iter() {
            handler();
        }
    }

    fn register(&mut self, path: String, vtable: &'static ComponentVTable<()>, attributes: Option<AttributeStore>) -> ComponentDesc {
        if let Some(vpath) = vtable.path {
            assert_eq!(path, vpath, "Static name does not match provided name");
        }

        let index = match self.component_paths.entry(path.to_owned()) {
            Entry::Occupied(slot) => *slot.get(),
            Entry::Vacant(slot) => {
                let index = self.components.len().try_into().expect("Maximum component count exceeded");
                slot.insert(index);

                let desc = ComponentDesc::new(index, vtable);

                // If a PrimitiveComponentType can be created from this component's type, create a PrimitiveComponent for it
                let primitive_component =
                    TYPE_ID_TO_PRIMITIVE_TYPE.get(&(vtable.get_type_id)()).copied().map(|ty| PrimitiveComponent { ty, desc });

                self.components.push(RegistryComponent { desc, primitive_component });

                index
            }
        };

        let slot = &mut self.components[index as usize];

        let mut dst = (vtable.attributes_init)(slot.desc);
        dst.set(ComponentPath(path));

        if let Some(src) = attributes {
            dst.append(&src);
        }

        slot.desc
    }

    pub(crate) fn register_external(
        &mut self,
        path: String,
        vtable: &'static ComponentVTable<()>,
        mut attributes: AttributeStore,
    ) -> ComponentDesc {
        assert_eq!(None, vtable.path, "Static name does not match provided name");

        log::debug!("Registering external component: {path}");

        attributes.set(External);
        self.register(path, vtable, Some(attributes))
    }

    pub fn register_static(&mut self, path: &'static str, vtable: &'static ComponentVTable<()>) -> ComponentDesc {
        log::debug!("Registering static component: {path}");
        self.register(path.into(), vtable, Default::default())
    }

    pub fn path_to_index(&self, path: &str) -> Option<u32> {
        self.component_paths.get(path).copied()
    }

    pub fn get_by_path(&self, path: &str) -> Option<ComponentDesc> {
        let index = *self.component_paths.get(path)?;
        Some(self.components[index as usize].desc)
    }

    pub fn get_by_index(&self, index: u32) -> Option<ComponentDesc> {
        self.components.get(index as usize).map(|b| b.desc)
    }

    pub fn get_primitive_component(&self, idx: u32) -> Option<PrimitiveComponent> {
        self.components.get(idx as usize).unwrap().primitive_component.clone()
    }

    /// Returns an iterator over all primitive components and their descs.
    pub fn all_primitive(&self) -> impl Iterator<Item = &PrimitiveComponent> + '_ {
        self.components.iter().filter_map(|v| Some(v.primitive_component.as_ref()?))
    }

    /// Returns an iterator over all primitive components that were externally defined and their descs.
    pub fn all_external(&self) -> impl Iterator<Item = ExternalComponentDesc> + '_ {
        self.all_primitive().filter(|pc| pc.desc.has_attribute::<External>()).map(|pc| ExternalComponentDesc {
            path: pc.desc.path(),
            ty: pc.ty,
            attributes: ExternalComponentAttributes::from_existing_component(pc.desc),
        })
    }

    pub fn all(&self) -> impl Iterator<Item = ComponentDesc> + '_ {
        self.components.iter().map(|v| v.desc)
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}
