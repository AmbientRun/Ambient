use core::fmt;
use std::{
    any::{Any, TypeId},
    cmp::Ordering,
    fmt::Debug,
    marker::PhantomData,
};

use parking_lot::MappedRwLockReadGuard;
use serde::{
    self,
    de::{self, DeserializeSeed},
    Deserialize, Serialize,
};

use crate::{
    component_traits::IComponentBuffer, with_component_registry, AttributeGuard,
    AttributeStoreGuard, AttributeStoreGuardMut, ComponentAttribute, ComponentEntry, ComponentPath,
    ComponentVTable, Debuggable, Description, Name, Serializable,
};

use ambient_shared_types::ComponentIndex;

pub trait ComponentValueBase: Send + Sync + as_any::AsAny {}

impl<T: Send + Sync + 'static> ComponentValueBase for T {}
pub trait ComponentValue: ComponentValueBase + Clone {}
impl<T: ComponentValueBase + Clone> ComponentValue for T {}

/// Implemented for component values that can be used as an enum
pub trait EnumComponent: Clone + Send + Sync {
    fn to_u32(&self) -> u32;
    fn from_u32(v: u32) -> Option<Self>
    where
        Self: Sized;
}
impl EnumComponent for u32 {
    fn to_u32(&self) -> u32 {
        *self
    }

    fn from_u32(v: u32) -> Option<Self>
    where
        Self: Sized,
    {
        Some(v)
    }
}

/// Component key
pub struct Component<T: 'static> {
    desc: ComponentDesc,
    _marker: PhantomData<T>,
}

impl<T: 'static> From<ComponentDesc> for Component<T> {
    fn from(value: ComponentDesc) -> Self {
        Self::new(value)
    }
}

impl<T: 'static> From<Component<T>> for ComponentDesc {
    #[inline]
    fn from(value: Component<T>) -> Self {
        value.desc
    }
}

impl<T: 'static> std::ops::Deref for Component<T> {
    type Target = ComponentDesc;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.desc
    }
}

impl<T> Debug for Component<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Component")
            .field("path", &self.path())
            .field("index", &self.desc.index)
            .finish()
    }
}

impl Debug for ComponentDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentDesc")
            .field("path", &self.path())
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

impl<T: 'static> Component<T> {
    /// Returns an untyped description of the component key
    #[inline]
    pub fn desc(&self) -> ComponentDesc {
        self.desc
    }
}

impl<T> Clone for Component<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Component<T> {}

impl<T> PartialEq for Component<T> {
    fn eq(&self, other: &Self) -> bool {
        self.desc.index == other.index
    }
}

impl<T> Eq for Component<T> {}

impl<T> PartialEq<ComponentDesc> for Component<T> {
    fn eq(&self, other: &ComponentDesc) -> bool {
        self.desc.index == other.index
    }
}

impl<T> PartialEq<Component<T>> for ComponentDesc {
    fn eq(&self, other: &Component<T>) -> bool {
        self.index == other.desc.index
    }
}

impl<T> PartialOrd for Component<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.desc.index.partial_cmp(&other.desc.index)
    }
}

impl<T> Ord for Component<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.desc.index.cmp(&other.desc.index)
    }
}

impl<T: ComponentValue> std::hash::Hash for Component<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index().hash(state);
    }
}

impl<T> Component<T> {
    /// Create a component key from the untyped description
    /// # Panics
    ///
    /// If the types do not match
    pub fn new(desc: ComponentDesc) -> Self {
        if !desc.is::<T>() {
            panic!(
                "Attempt to convert component description of {:?} into component of type {:?}",
                desc.type_name(),
                std::any::type_name::<T>()
            );
        }
        Self {
            desc,
            _marker: PhantomData,
        }
    }

    pub fn as_debug<'a>(&self, value: &'a T) -> &'a dyn Debug {
        self.desc.as_debug(value)
    }
}

/// Contains enough information to construct, erase, and de-erase a component key.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComponentDesc {
    index: ComponentIndex,
    pub(crate) vtable: &'static ComponentVTable<()>,
}

impl Eq for ComponentDesc {}

impl std::hash::Hash for ComponentDesc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl PartialEq for ComponentDesc {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for ComponentDesc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl Ord for ComponentDesc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl ComponentDesc {
    /// The fully qualified component path.
    pub fn path(&self) -> String {
        if let Some(path) = self.vtable.path {
            path.to_string()
        } else {
            self.attribute::<ComponentPath>()
                .expect("No path for component")
                .0
                .clone()
        }
    }

    /// The last segment of the component path. Do not use this as a unique identifier; use [Self::path] instead.
    pub fn path_last(&self) -> String {
        if let Some(path) = self.vtable.path {
            path.rsplit_once("::").map(|v| v.1).unwrap_or(path).into()
        } else {
            let path = &self
                .attribute::<ComponentPath>()
                .expect("No path for component")
                .0;
            path.rsplit_once("::").map(|v| v.1).unwrap_or(path).into()
        }
    }

    /// A human-friendly name, if available. Corresponds to the [Name] attribute.
    pub fn name(&self) -> Option<String> {
        Some(self.attribute::<Name>()?.0.clone())
    }

    /// A human-friendly description, if available. Corresponds to the [Description] attribute.
    pub fn description(&self) -> Option<String> {
        Some(self.attribute::<Description>()?.0.clone())
    }

    pub fn type_name(&self) -> &'static str {
        (self.vtable.get_type_name)()
    }

    pub fn type_id(&self) -> TypeId {
        (self.vtable.get_type_id)()
    }

    #[inline]
    /// Returns true if the entry is of type `T`
    pub fn is<T: 'static>(&self) -> bool {
        (self.vtable.get_type_id)() == TypeId::of::<T>()
    }

    pub(crate) fn new(index: ComponentIndex, vtable: &'static ComponentVTable<()>) -> Self {
        Self { index, vtable }
    }

    pub fn has_attribute<A: ComponentAttribute>(&self) -> bool {
        (self.vtable.attributes)(*self).has::<A>()
    }

    pub fn attribute<A: ComponentAttribute>(&self) -> Option<AttributeGuard<A>> {
        let guard = (self.vtable.attributes)(*self);
        MappedRwLockReadGuard::try_map(guard, |store| store.get::<A>()).ok()
    }

    pub fn attributes(&self) -> AttributeStoreGuard {
        (self.vtable.attributes)(*self)
    }

    pub fn attributes_mut(&self) -> AttributeStoreGuardMut {
        (self.vtable.attributes_init)(*self)
    }

    pub fn as_debug<'a>(&self, value: &'a dyn Any) -> &'a dyn Debug {
        struct NoDebug;
        impl Debug for NoDebug {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("...")
            }
        }

        if let Some(v) = self.attribute::<Debuggable>() {
            v.as_debug(value)
        } else {
            &NoDebug
        }
    }

    pub fn index(&self) -> ComponentIndex {
        self.index
    }

    pub fn from_json(&self, value: &str) -> Result<ComponentEntry, serde_json::Error> {
        self.attribute::<Serializable>()
            .expect("Component is not serializable")
            .deserializer(*self)
            .deserialize(&mut serde_json::de::Deserializer::from_str(value))
    }

    /// Converts the **value** to json
    pub fn to_json(&self, value: &ComponentEntry) -> Result<String, serde_json::Error> {
        serde_json::to_string(
            self.attribute::<Serializable>()
                .expect("Component is not serializable")
                .serialize(value),
        )
    }

    pub fn create_buffer(&self) -> Box<dyn IComponentBuffer> {
        (self.vtable.impl_create_buffer)(*self)
    }
}

impl Serialize for ComponentDesc {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.path())
    }
}

impl<'de> Deserialize<'de> for ComponentDesc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BoxIComponentVisitor;

        impl<'de> serde::de::Visitor<'de> for BoxIComponentVisitor {
            type Value = ComponentDesc;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ComponentDesc")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let component = with_component_registry(|r| r.get_by_path(v));
                match component {
                    Some(comp) => Ok(comp),
                    None => Err(de::Error::custom(format!("No such component: {v}"))),
                }
            }
        }

        deserializer.deserialize_str(BoxIComponentVisitor)
    }
}

#[macro_export]
/// Defines components to use within the ECS.
///
/// If the type is captured by [crate::PrimitiveComponent] and has a [crate::Name] and [crate::Description], these will be accessible by wasm.
/// Please make sure to update the interface components if you update WASM-visible components.
macro_rules! components {
    ($ns: literal, { $($(#[$outer:meta])* $(@[$($attr: ty$([ $params: expr ])?),*])? $vis: vis $name:ident: $ty:ty,)*}) => {
        $(
            $crate::paste::paste! {
                #[allow(non_upper_case_globals)]
                #[doc(hidden)]
                static [<comp_ $name>]: $crate::OnceCell<$crate::ComponentDesc> = $crate::OnceCell::new();

                #[doc(hidden)]
                fn [< __init_component_ $name>] (reg: &mut $crate::ComponentRegistry) -> $crate::ComponentDesc {
                    fn init_attr(_component: $crate::Component<$ty>) -> $crate::parking_lot::RwLock<$crate::AttributeStore> {

                        #[allow(unused_mut)]
                        let mut store = $crate::AttributeStore::new();


                        $( $(
                            <$attr as $crate::AttributeConstructor::<$ty, _>>::construct(
                                &mut store,
                                ($($params),*)
                            );
                        )*)*

                        $crate::parking_lot::RwLock::new(store)

                    }

                    static ATTRIBUTES: $crate::OnceCell<$crate::parking_lot::RwLock<$crate::AttributeStore>> = $crate::OnceCell::new();

                    static PATH: &str = concat!("ambient_core::", $ns, "::", stringify!($name));
                    static VTABLE: &$crate::ComponentVTable<$ty> = &$crate::ComponentVTable::construct(
                        PATH,
                        |desc| $crate::parking_lot::RwLockReadGuard::map(ATTRIBUTES.get_or_init(|| init_attr($crate::Component::new(desc))).read(), |v| v),
                        |desc| $crate::parking_lot::RwLockWriteGuard::map(ATTRIBUTES.get_or_init(|| init_attr($crate::Component::new(desc))).write(), |v| v)
                    );

                    *[<comp_ $name>].get_or_init(|| {
                        reg.register_static(PATH, unsafe { VTABLE.erase() } )
                    })
                }

                $(#[$outer])*
                pub fn $name() -> $crate::Component<$ty> {

                    let desc = *[<comp_ $name>].get().unwrap_or_else(|| {
                        panic!("Component {} is not initialized", concat!("ambient_core::", $ns, "::", stringify!($name)))
                    });

                    $crate::Component::new(desc)
                }
            }
        )*


        /// Initialize the components for the module
        pub fn init_components() {
                let mut reg = $crate::ComponentRegistry::get_mut();
                $(
                    $crate::paste::paste! {
                        [< __init_component_ $name>](&mut reg);
                    }
                )*
        }

    }
}

#[cfg(test)]
mod test {
    use std::{ptr, sync::Arc};

    use once_cell::sync::Lazy;
    use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
    use serde::de::DeserializeSeed;

    use super::*;
    use crate::{AttributeStore, ComponentVTable, MakeDefault, Networked, Store};

    #[test]
    fn manual_component() {
        static ATTRIBUTES: Lazy<RwLock<AttributeStore>> = Lazy::new(Default::default);
        static VTABLE: &ComponentVTable<String> = &ComponentVTable::construct(
            "core::test::my_component",
            |_| RwLockReadGuard::map(ATTRIBUTES.read(), |v| v),
            |_| RwLockWriteGuard::map(ATTRIBUTES.write(), |v| v),
        );

        let component: Component<String> =
            Component::new(ComponentDesc::new(1, unsafe { VTABLE.erase() }));

        let value = ComponentEntry::new(component, "Hello, World".into());

        // Clone is for purpose of comparison
        #[allow(clippy::redundant_clone)]
        let value2 = value.clone();

        let s = value.downcast_ref::<String>();
        let s2 = value2.downcast_ref::<String>();

        // Since they are cloned, they should not be reference equal
        assert!(!ptr::eq(s as *const String, s2 as *const String));
        // They are however value equal
        assert_eq!(
            value.downcast_ref::<String>(),
            value2.downcast_ref::<String>()
        );

        assert_eq!(value.try_downcast_ref::<&str>(), None);
    }

    #[derive(PartialEq, Eq, Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Person {
        name: String,
        age: i32,
    }

    #[test]
    fn component_macro() {
        components! ("component_macro",{
            @[Serializable, Debuggable]
            foo: String,
            /// This is a person
            @[Serializable, Debuggable]
            person: Person,
        });

        init_components();

        let component = foo();

        assert_eq!(component.path_last(), "foo");

        assert!(component.has_attribute::<Serializable>());

        let p = Person {
            name: "Adam".into(),
            age: 28,
        };
        let entry = ComponentEntry::new(person(), p);

        let str = serde_json::to_string_pretty(
            entry.attribute::<Serializable>().unwrap().serialize(&entry),
        )
        .unwrap();

        eprintln!("Serialized: {str}");

        let ser = person().attribute::<Serializable>().unwrap();

        let value: ComponentEntry = ser
            .deserializer(person().desc())
            .deserialize(&mut serde_json::Deserializer::from_str(&str))
            .unwrap();

        eprintln!("Value: {:?}", value.as_debug());

        let p = entry.into_inner::<Person>();
        assert_eq!(value.downcast_ref::<Person>(), &p);
        assert_eq!(value.try_downcast_ref::<String>(), None);
    }

    #[test]
    fn make_default() {
        fn default_person() -> Person {
            Person {
                age: 21,
                name: "unnamed".into(),
            }
        }

        components! ("make_default", {
            @[MakeDefault, Debuggable]
            people: Vec<Person>,
            @[MakeDefault[default_person], Debuggable, Store, Networked]
            person: Person,
        });
        init_components();

        let people_desc: ComponentDesc = people().desc();
        let person_desc: ComponentDesc = person().desc();

        let mut people = people_desc
            .attribute::<MakeDefault>()
            .unwrap()
            .make_default(people_desc);

        let mut person = person_desc
            .attribute::<MakeDefault>()
            .unwrap()
            .make_default(person_desc);

        assert_eq!(
            person.downcast_ref::<Person>(),
            &Person {
                age: 21,
                name: "unnamed".into()
            }
        );

        people
            .downcast_mut::<Vec<Person>>()
            .push(person.downcast_cloned::<Person>());

        assert_eq!(
            &people.downcast_mut::<Vec<Person>>()[0],
            person.downcast_ref::<Person>()
        );
        person.downcast_mut::<Person>().name = "Smith".to_string();
        assert_ne!(
            &people.downcast_mut::<Vec<Person>>()[0],
            person.downcast_ref::<Person>()
        );

        eprintln!("people: {people:?}, person: {person:?}");
    }

    #[test]
    fn test_take() {
        components! ("test_take", {
            @[Store]
            my_component: Arc<String>,
        });

        init_components();
        let shared = Arc::new("Foo".to_string());

        {
            let value = ComponentEntry::new(my_component(), shared.clone());
            let value2 = ComponentEntry::new(my_component(), shared.clone());

            assert_eq!(Arc::strong_count(&shared), 3);
            drop(value);
            assert_eq!(Arc::strong_count(&shared), 2);

            let value = value2.into_inner::<Arc<String>>();
            assert_eq!(Arc::strong_count(&shared), 2);
            drop(value);
            assert_eq!(Arc::strong_count(&shared), 1);
        }

        assert_eq!(Arc::strong_count(&shared), 1);
    }

    #[test]
    fn leak_test() {
        let shared = Arc::new("Foo".to_string());

        components! ("leak_test", {
            my_component: Arc<String>,
        });

        init_components();

        {
            let value = ComponentEntry::new(my_component(), shared.clone());
            let value2 = ComponentEntry::new(my_component(), shared.clone());

            assert_eq!(Arc::strong_count(&shared), 3);
            drop(value);
            assert_eq!(Arc::strong_count(&shared), 2);

            #[allow(clippy::redundant_clone)]
            let value3 = value2.clone();
            let value4: Arc<String> = value2.downcast_cloned();
            assert_eq!(Arc::strong_count(&shared), 4);
            drop(value4);
            assert_eq!(Arc::strong_count(&shared), 3);

            assert_eq!(value3.downcast_ref::<Arc<String>>(), &shared);
            assert_eq!(value3.downcast_ref::<Arc<String>>(), value2.downcast_ref());

            assert!(!ptr::eq(
                value3.downcast_ref::<Arc<String>>() as *const Arc<String>,
                &shared as *const _
            ));
            assert!(!ptr::eq(
                value3.downcast_ref::<Arc<String>>() as *const Arc<String>,
                value2.downcast_ref::<Arc<String>>() as *const _
            ));

            assert!(ptr::eq(
                &**value3.downcast_ref::<Arc<String>>() as *const String,
                &*shared as *const _
            ));
        }

        assert_eq!(Arc::strong_count(&shared), 1);
    }
}
