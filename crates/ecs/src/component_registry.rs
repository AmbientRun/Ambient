use std::{
    any::TypeId, collections::{hash_map::Entry, HashMap}
};

use elements_std::{asset_url::AbsAssetUrl, events::EventDispatcher};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use super::*;
use crate::ComponentVTable;

static COMPONENT_REGISTRY: Lazy<RwLock<ComponentRegistry>> = Lazy::new(|| RwLock::new(ComponentRegistry::default()));
pub fn with_component_registry<R>(f: impl FnOnce(&ComponentRegistry) -> R + Sync + Send) -> R {
    let lock = COMPONENT_REGISTRY.read();
    f(&lock)
}

pub(crate) struct RegistryComponent {
    pub(crate) desc: ComponentDesc,
    pub(crate) primitive_component_type: Option<PrimitiveComponentType>,
    pub(crate) primitive_component: Option<PrimitiveComponent>,
    /// Some if there are external attributes.
    ///
    /// Othewise, attributes are stored statically
    attributes: Option<HashMap<TypeId, AttributeEntry>>,
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
    pub fn get() -> parking_lot::RwLockReadGuard<'static, Self> {
        COMPONENT_REGISTRY.read()
    }
    pub fn get_mut() -> parking_lot::RwLockWriteGuard<'static, Self> {
        COMPONENT_REGISTRY.write()
    }
    /// When decorating is true, the components read from the source will be assumed to already exist and we'll just add
    /// metadata to them
    pub fn add_external(&mut self, source: AbsAssetUrl, decorating: bool) {
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
        self.add_external_from_iterator(components.into_iter().map(|c| (c.id, c.type_)), decorating)
    }
    /// When decorating is true, the components read from the source will be assumed to already exist and we'll just add
    /// metadata to them
    pub fn add_external_from_iterator(&mut self, components: impl Iterator<Item = (String, PrimitiveComponentType)>, decorating: bool) {
        for (id, type_) in components {
            type_.register(self, &id, decorating);
        }

        for handler in self.on_external_components_change.iter() {
            handler();
        }
    }

    pub fn register_external(
        &mut self,
        path: String,
        vtable: &'static ComponentVTable<()>,
        attributes: impl IntoIterator<Item = Box<dyn Fn(ComponentDesc) -> AttributeEntry>>,
    ) -> ComponentDesc {
        assert_eq!(None, vtable.path, "Static name does not match provided name");

        log::info!("Registering component: {path}");

        let index = self.components.len().try_into().expect("Maximum component reached");
        let desc = ComponentDesc::new(index, vtable);

        let attributes = attributes.into_iter();
        let attributes =
            attributes.map(|v| v(desc)).chain([AttributeEntry::from_value(ComponentPath(path.clone()))]).map(|v| (v.key(), v)).collect();

        self.components.push(RegistryComponent {
            desc,
            primitive_component_type: None,
            primitive_component: None,
            attributes: Some(attributes),
        });

        match self.component_paths.entry(path) {
            Entry::Occupied(slot) => panic!("Duplicate component path: {:?}", slot.key()),
            Entry::Vacant(slot) => slot.insert(index),
        };

        desc
    }

    pub fn register_static(&mut self, path: &'static str, vtable: &'static ComponentVTable<()>) -> ComponentDesc {
        assert_eq!(Some(path), vtable.path, "Static name does not match provided name");

        log::info!("Registering component: {path}");

        let index = self.components.len().try_into().expect("Maximum component reached");
        let desc = ComponentDesc::new(index, vtable);

        self.components.push(RegistryComponent {
            desc,
            primitive_component_type: None,
            primitive_component: None,
            attributes: Default::default(),
        });

        match self.component_paths.entry(path.into()) {
            Entry::Occupied(slot) => panic!("Duplicate component path: {:?}", slot.key()),
            Entry::Vacant(slot) => slot.insert(index),
        };

        desc
    }

    /// Sets the primitive component for an existing component
    pub fn set_primitive_component(&mut self, path: &str, ty: PrimitiveComponentType) -> PrimitiveComponent {
        let index = *match self.component_paths.get(path) {
            Some(v) => v,
            None => {
                panic!("Attempt to set primitive type for unknown component: {path:?}");
            }
        };

        let entry = &mut self.components[index as usize];

        let prim = PrimitiveComponent { ty: ty.clone(), desc: entry.desc };
        entry.primitive_component_type = Some(ty);
        entry.primitive_component = Some(prim.clone());
        prim
    }

    pub(crate) fn get_external_attribute(&self, index: u32, key: TypeId) -> Option<AttributeEntry> {
        let entry = &self.components[index as usize];

        entry.attributes.as_ref().expect("No external attributes on static components").get(&key).cloned()
    }

    pub fn path_to_index(&self, path: &str) -> Option<u32> {
        self.component_paths.get(path).copied()
    }

    pub fn get_by_path(&self, path: &str) -> Option<ComponentDesc> {
        let index = *self.component_paths.get(path)?;
        Some(self.components[index as usize].desc)
    }

    pub fn register<T: ComponentValue>(&mut self, namespace: &str, name: &str, component: &mut Component<T>) {
        todo!()
        // if component.index >= 0 {
        //     return;
        // }
        // self.register_with_id(&format!("{namespace}::{name}"), component, false, None, None);
    }

    // pub(crate) fn register_with_id(
    //     &mut self,
    //     id: &str,
    //     component: &mut dyn IComponent,
    //     decorating: bool,
    //     primitive_component_type: Option<PrimitiveComponentType>,
    //     mut primitive_component: Option<PrimitiveComponent>,
    // ) {
    //     todo!()
    //     // if let Some(idx) = self.name_to_idx.get(id) {
    //     //     if decorating {
    //     //         component.set_index(*idx);

    //     //         if let Some(primitive_component) = &mut primitive_component {
    //     //             primitive_component.as_component_mut().set_index(*idx);
    //     //         }

    //     //         self.components[*idx].primitive_component_type = primitive_component_type;
    //     //         self.components[*idx].primitive_component = primitive_component;
    //     //     } else {
    //     //         log::warn!("Duplicate components: {}", id);
    //     //     }
    //     //     return;
    //     // }
    //     // let index = self.components.len();
    //     // component.set_index(index);
    //     // if let Some(primitive_component) = &mut primitive_component {
    //     //     primitive_component.as_component_mut().set_index(index);
    //     // }
    //     // let reg_comp = RegistryComponent { component: component.clone_boxed(), primitive_component_type, primitive_component };
    //     // self.components.push(reg_comp.clone());
    //     // self.name_to_idx.insert(id.to_owned(), index);
    //     // self.idx_to_id.insert(component.get_index(), id.to_owned());
    // }

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

// #[macro_export]
// macro_rules! components {
//     ( $namespace:literal, { $( $(#[$outer:meta])* $name:ident : $ty:ty, )+ } ) => {
//         $(
//             $crate::paste::paste! {
//                 #[allow(non_upper_case_globals)]
//                 #[no_mangle]
//                 static mut [<comp_ $name>]: $crate::Component<$ty> = $crate::Component::new_with_name(-1, stringify!($name));
//             }
//             $(#[$outer])*
//             pub fn $name() -> $crate::Component<$ty> {
//                 $crate::paste::paste! {
//                     unsafe { [<comp_ $name>] }
//                 }
//             }
//         )*
//         /// Initialize the components for the module
//         static COMPONENTS_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
//         pub fn init_components() {
//             use std::sync::atomic::Ordering;

//             if COMPONENTS_INITIALIZED.load(Ordering::SeqCst) {
//                 return;
//             }

//             unsafe {
//                 $(
//                     $crate::paste::paste! {
//                         $crate::ComponentRegistry::get_mut().register(concat!("core::", $namespace), stringify!($name), &mut [<comp_ $name>]);
//                     }
//                 )*
//             }
//             COMPONENTS_INITIALIZED.store(true, Ordering::SeqCst);
//         }
//     };
// }
