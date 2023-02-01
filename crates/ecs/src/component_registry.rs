use std::collections::{hash_map::Entry, BTreeMap, HashMap};

use elements_std::{asset_url::AbsAssetUrl, events::EventDispatcher};
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
    pub(crate) primitive_component_type: Option<PrimitiveComponentType>,
    pub(crate) primitive_component: Option<PrimitiveComponent>,
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
    /// When decorating is true, the components read from the source will be assumed to already exist and we'll just add
    /// metadata to them
    pub fn add_external(&mut self, source: AbsAssetUrl) {
        let data: Vec<u8> = if let Some(path) = source.to_file_path().unwrap() {
            std::fs::read(path).unwrap()
        } else {
            reqwest::blocking::get(source.0).unwrap().bytes().unwrap().into()
        };

        #[derive(Deserialize)]
        struct Entry {
            id: String,
            #[serde(rename = "type")]
            type_: PrimitiveComponentType,
        }
        let components: Vec<Entry> = serde_json::from_slice(&data).unwrap();
        self.add_external_from_iterator(components.into_iter().map(|c| (c.id, c.type_)))
    }
    /// When decorating is true, the components read from the source will be assumed to already exist and we'll just add
    /// metadata to them
    pub fn add_external_from_iterator(&mut self, components: impl Iterator<Item = (String, PrimitiveComponentType)>) {
        for (id, type_) in components {
            type_.register(self, &id);
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

                self.components.push(RegistryComponent { desc, primitive_component_type: None, primitive_component: None });

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

    pub fn register_external(&mut self, path: String, vtable: &'static ComponentVTable<()>, attributes: AttributeStore) -> ComponentDesc {
        assert_eq!(None, vtable.path, "Static name does not match provided name");

        log::debug!("Registering external component: {path}");

        self.register(path, vtable, Some(attributes))
    }

    pub fn register_static(&mut self, path: &'static str, vtable: &'static ComponentVTable<()>) -> ComponentDesc {
        log::debug!("Registering static component: {path}");
        self.register(path.into(), vtable, Default::default())
    }

    /// Sets the primitive component for an existing component
    pub fn set_primitive_component(&mut self, path: &str, ty: PrimitiveComponentType) -> Option<PrimitiveComponent> {
        let index = *match self.component_paths.get(path) {
            Some(v) => v,
            None => {
                log::error!("Attempt to set primitive type for unknown component: {path:?}");
                return None;
            }
        };

        let entry = &mut self.components[index as usize];

        let prim = PrimitiveComponent { ty: ty.clone(), desc: entry.desc };
        entry.primitive_component_type = Some(ty);
        entry.primitive_component = Some(prim.clone());

        // TODO: externally defined attributes
        // // Hydrate the store with the primitive component attributes
        // if let Some(src) = PRIMITIVE_ATTRIBUTE_REGISTRY.read().get(&ty) {
        //     let mut dst = (entry.desc.vtable.attributes_init)(entry.desc);
        //     log::info!("Hydrating {:?}", path);
        //     dst.append(src)
        // } else {
        //     log::warn!("No primitive attributes for {ty:?}");
        // }

        Some(prim)
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

    pub fn all_external(&self) -> impl Iterator<Item = ComponentDesc> + '_ {
        self.components.iter().filter(|v| v.primitive_component_type.is_some()).map(|x| x.desc)
    }

    pub fn all(&self) -> impl Iterator<Item = ComponentDesc> + '_ {
        self.components.iter().map(|v| v.desc)
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}
