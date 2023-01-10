use std::collections::HashMap;

use elements_std::{asset_url::AbsAssetUrl, events::EventDispatcher};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use super::*;

static COMPONENT_REGISTRY: Lazy<RwLock<ComponentRegistry>> = Lazy::new(|| RwLock::new(ComponentRegistry::default()));
pub fn with_component_registry<R>(f: impl FnOnce(&ComponentRegistry) -> R + Sync + Send) -> R {
    let lock = COMPONENT_REGISTRY.read();
    f(&lock)
}
pub fn with_component_registry_mut<R>(f: impl FnOnce(&mut ComponentRegistry) -> R + Sync + Send) -> R {
    let mut lock = COMPONENT_REGISTRY.write();
    f(&mut lock)
}

#[derive(Clone)]
pub(crate) struct RegistryComponent {
    pub(crate) component: Box<dyn IComponent>,
    pub(crate) primitive_component_type: Option<PrimitiveComponentType>,
    pub(crate) primitive_component: Option<PrimitiveComponent>,
}

#[derive(Clone, Default)]
pub struct ComponentRegistry {
    pub(crate) name_to_idx: HashMap<String, usize>,
    pub(crate) idx_to_id: HashMap<usize, String>,
    pub(crate) components: Vec<RegistryComponent>,

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
        for Entry { id, type_ } in components {
            type_.register(self, &id, decorating);
        }
        for handler in self.on_external_components_change.iter() {
            handler();
        }
    }
    pub fn register<T: ComponentValue>(&mut self, namespace: &str, name: &str, component: &mut Component<T>) {
        if component.index >= 0 {
            return;
        }
        self.register_with_id(&format!("{namespace}::{name}"), component, false, None, None);
    }
    pub(crate) fn register_with_id(
        &mut self,
        id: &str,
        component: &mut dyn IComponent,
        decorating: bool,
        primitive_component_type: Option<PrimitiveComponentType>,
        mut primitive_component: Option<PrimitiveComponent>,
    ) {
        if let Some(idx) = self.name_to_idx.get(id) {
            if decorating {
                component.set_index(*idx);

                if let Some(primitive_component) = &mut primitive_component {
                    primitive_component.as_component_mut().set_index(*idx);
                }

                self.components[*idx].primitive_component_type = primitive_component_type;
                self.components[*idx].primitive_component = primitive_component;
            } else {
                log::warn!("Duplicate components: {}", id);
            }
            return;
        }
        let index = self.components.len();
        component.set_index(index);
        if let Some(primitive_component) = &mut primitive_component {
            primitive_component.as_component_mut().set_index(index);
        }
        let reg_comp = RegistryComponent { component: component.clone_boxed(), primitive_component_type, primitive_component };
        self.components.push(reg_comp.clone());
        self.name_to_idx.insert(id.to_owned(), index);
        self.idx_to_id.insert(component.get_index(), id.to_owned());
    }

    pub fn get_by_id(&mut self, id: &str) -> Option<&dyn IComponent> {
        self.name_to_idx.get(id).map(|b| self.components[*b].component.as_ref())
    }
    pub fn get_by_index(&self, index: usize) -> Option<&dyn IComponent> {
        self.components.get(index).map(|b| b.component.as_ref())
    }
    pub fn get_by_index_type<T: IComponent + Clone>(&self, index: usize) -> Option<T> {
        self.get_by_index(index)?.downcast_ref().cloned()
    }
    pub fn get_id_for_opt(&self, component: &dyn IComponent) -> Option<&str> {
        self.idx_to_id().get(&component.get_index()).map(|s| s.as_str())
    }
    pub fn get_primitive_component(&self, idx: usize) -> Option<PrimitiveComponent> {
        self.components.get(idx).unwrap().primitive_component
    }
    /// Will panic if the specified component does not exist
    pub fn get_id_for(&self, component: &dyn IComponent) -> &str {
        match self.get_id_for_opt(component) {
            Some(id) => id,
            None => panic!("failed to get id for component {}", component.get_index()),
        }
    }
    pub fn all_external(&self) -> impl Iterator<Item = &Box<dyn IComponent>> {
        self.components.iter().filter(|x| x.primitive_component_type.is_some()).map(|x| &x.component)
    }
    pub fn all(&self) -> impl Iterator<Item = &Box<dyn IComponent>> {
        self.components.iter().map(|x| &x.component)
    }
    pub fn idx_to_id(&self) -> &HashMap<usize, String> {
        &self.idx_to_id
    }
    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}

#[macro_export]
macro_rules! components {
    ( $namespace:literal, { $( $(#[$outer:meta])* $name:ident : $ty:ty, )+ } ) => {
        $(
            $crate::paste::paste! {
                #[allow(non_upper_case_globals)]
                #[no_mangle]
                static mut [<comp_ $name>]: $crate::Component<$ty> = $crate::Component::new_with_name(-1, stringify!($name));
            }
            $(#[$outer])*
            pub fn $name() -> $crate::Component<$ty> {
                $crate::paste::paste! {
                    unsafe { [<comp_ $name>] }
                }
            }
        )*
        /// Initialize the components for the module
        static COMPONENTS_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        pub fn init_components() {
            use std::sync::atomic::Ordering;

            if COMPONENTS_INITIALIZED.load(Ordering::SeqCst) {
                return;
            }

            $crate::with_component_registry_mut(|registry| unsafe {
                $(
                    $crate::paste::paste! {
                        registry.register(concat!("core::", $namespace), stringify!($name), &mut [<comp_ $name>]);
                    }
                )*
            });
            COMPONENTS_INITIALIZED.store(true, Ordering::SeqCst);
        }
    };
}
