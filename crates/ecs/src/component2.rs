use core::fmt;
use std::{
    any::{Any, TypeId}, cmp::Ordering, fmt::Debug, marker::PhantomData, mem::{self, ManuallyDrop, MaybeUninit}
};

use downcast_rs::Downcast;
use serde::{
    self, de::{self, DeserializeSeed}, Deserialize, Serialize
};

use crate::{
    component::{ComponentBuffer, IComponentBuffer}, with_component_registry, ComponentEntry, ComponentHolder, ComponentVTable, ErasedHolder
};

pub trait ComponentValueBase: Send + Sync + Downcast + 'static {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T: Send + Sync + 'static> ComponentValueBase for T {}
pub trait ComponentValue: ComponentValueBase + Clone {}
impl<T: ComponentValueBase + Clone> ComponentValue for T {}

/// Declares an attribute type which can be attached to a component
pub trait ComponentAttribute: 'static {
    type Value: 'static + Send + Sync;
}

macro_rules! component_attributes {
    ($($(#[$outer: meta])* $name: ident: $ty: ty,)*) => {
$(
        /// Component attribute
        $(#[$outer])*
        #[derive(Default, Eq, PartialEq, PartialOrd, Hash, Debug, Clone)]
        pub struct $name {}

        impl $crate::ComponentAttribute for $name {
            type Value = $ty;
        }

)        *
    };
}

pub trait ComponentAttributeValue<T, P> {
    /// Construct a new instance of the attribute value
    fn construct(component: Component<T>, params: P) -> Self;
}

/// Allow serializing a component entry
#[derive(Clone, Copy)]
pub struct ComponentSerializer {
    ser: fn(&ComponentEntry) -> &dyn erased_serde::Serialize,
    deser: fn(ComponentDesc, &mut dyn erased_serde::Deserializer) -> Result<ComponentEntry, erased_serde::Error>,
    desc: ComponentDesc,
}

impl<T: ComponentValue + Serialize + for<'de> Deserialize<'de>> ComponentAttributeValue<T, ()> for ComponentSerializer {
    fn construct(component: Component<T>, _: ()) -> Self {
        Self {
            ser: |v| v.downcast_ref::<T>() as &dyn erased_serde::Serialize,
            deser: |desc, deserializer| {
                let value = T::deserialize(deserializer)?;
                let entry = ComponentEntry::from_raw_parts(desc, value);
                Ok(entry)
            },
            desc: component.desc,
        }
    }
}

impl ComponentSerializer {
    /// Serialize a value
    pub fn serialize<'a>(&self, entry: &'a ComponentEntry) -> &'a dyn erased_serde::Serialize {
        (self.ser)(entry)
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for ComponentSerializer {
    type Value = ComponentEntry;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
        let deserializer = &mut deserializer;
        (self.deser)(self.desc, deserializer).map_err(serde::de::Error::custom)
    }
}

pub struct ComponentDebugger {
    debug: fn(&dyn Any) -> &dyn Debug,
}

impl<T> ComponentAttributeValue<T, ()> for ComponentDebugger
where
    T: Debug,
{
    fn construct(_: Component<T>, _: ()) -> Self {
        Self { debug: |entry| entry.downcast_ref::<T>().unwrap() as &dyn Debug }
    }
}

pub struct ComponentDefault {
    make_default: Box<dyn Fn() -> ComponentEntry + Send + Sync>,
}

impl ComponentDefault {
    /// Construct the default value of this component
    pub fn make_default(&self) -> ComponentEntry {
        (self.make_default)()
    }
}

impl<T: ComponentValue + Default> ComponentAttributeValue<T, ()> for ComponentDefault {
    fn construct(component: Component<T>, _: ()) -> Self {
        Self { make_default: Box::new(move || ComponentEntry::new(component, T::default())) }
    }
}

impl<T: ComponentValue, F: 'static + Send + Sync + Fn() -> T> ComponentAttributeValue<T, F> for ComponentDefault {
    fn construct(component: Component<T>, func: F) -> Self {
        Self { make_default: Box::new(move || ComponentEntry::new(component, func())) }
    }
}

component_attributes! {
    /// Declares a component as [`serde::Serialize`] and [`serde::Deserialize`]
    Serializable: ComponentSerializer,
    Debuggable: ComponentDebugger,
    MakeDefault: ComponentDefault,
    Store: (),
    Networked: (),
}

impl<T> ComponentAttributeValue<T, ()> for () {
    fn construct(_: Component<T>, _: ()) -> Self {}
}

pub struct AttributeEntry {
    /// const fn TypeId::of is not stable
    key: fn() -> TypeId,
    /// To use: cast to the correct type, which is determined by the attribute in use.
    ///
    /// It is recommended to use helper functions
    value: Box<dyn Any + Send + Sync>,
}

impl AttributeEntry {
    pub fn new<Attr, T, P>(component: Component<T>, params: P) -> Self
    where
        Attr: ComponentAttribute,
        Attr::Value: ComponentAttributeValue<T, P>,
        T: 'static,
    {
        Self { key: || TypeId::of::<Attr>(), value: Box::new(Attr::Value::construct(component, params)) as Box<dyn Any + Send + Sync> }
    }
}

/// Construct attributes from a slice
#[inline(always)]
pub fn slice_attrs(attrs: &'static [AttributeEntry], key: TypeId) -> Option<&'static AttributeEntry> {
    attrs.iter().find(|v| (v.key)() == key)
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
        f.debug_struct("Component").field("name", &self.desc.vtable.component_name).field("index", &self.desc.index).finish()
    }
}

impl Debug for ComponentDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentDesc").field("path", &self.path()).field("index", &self.index).finish_non_exhaustive()
    }
}

impl<T: 'static> Component<T> {
    pub fn name(&self) -> &'static str {
        self.desc.vtable.component_name
    }

    /// Returns an untyped description of the component key
    #[inline]
    pub fn desc(&self) -> ComponentDesc {
        self.desc
    }
}

impl<T> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self { desc: self.desc, _marker: PhantomData }
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
        Self { desc, _marker: PhantomData }
    }

    pub fn attribute<A: ComponentAttribute>(&self) -> Option<&'static A::Value> {
        self.desc.attribute::<A>()
    }

    pub fn as_debug<'a>(&self, value: &'a T) -> &'a dyn Debug {
        self.desc.as_debug(value)
    }
}

/// Contains enough information to construct, erase, and de-erase a component key.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComponentDesc {
    index: u32,
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
    /// Returns the fully qualified component path
    pub fn path(&self) -> String {
        self.vtable.path()
    }

    pub fn name(&self) -> &'static str {
        self.vtable.component_name
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

    pub(crate) fn new(index: u32, vtable: &'static ComponentVTable<()>) -> Self {
        Self { index, vtable }
    }

    pub fn attribute<A: ComponentAttribute>(&self) -> Option<&'static A::Value> {
        let entry = (self.vtable.custom_attrs)(*self, TypeId::of::<A>())?;
        let value = entry.value.downcast_ref().expect("Mismatched attribute types");
        Some(value)
    }

    pub fn as_debug<'a>(&self, value: &'a dyn Any) -> &'a dyn Debug {
        struct NoDebug;
        impl Debug for NoDebug {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("...")
            }
        }

        if let Some(v) = self.attribute::<Debuggable>() {
            (v.debug)(value)
        } else {
            &NoDebug
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn from_json(&self, value: &str) -> Result<ComponentEntry, serde_json::Error> {
        self.attribute::<Serializable>()
            .expect("Component is not serializable")
            .deserialize(&mut serde_json::de::Deserializer::from_str(value))
    }

    /// Converts the **value** to json
    pub fn to_json(&self, value: &ComponentEntry) -> Result<String, serde_json::Error> {
        serde_json::to_string(self.attribute::<Serializable>().expect("Component is not serializable").serialize(value))
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
macro_rules! components {
    ($ns: literal, { $($(#[$outer:meta])* $(@[$($attr: ty$([ $params: tt ])?),*])? $vis: vis $name:ident: $ty:ty,)*}) => {
        $(
            $crate::paste::paste! {
                #[allow(non_upper_case_globals)]
                #[doc(hidden)]
                static [<comp_ $name>]: $crate::OnceCell<$crate::ComponentDesc> = $crate::OnceCell::new();

                #[doc(hidden)]
                fn [< __init_component_ $name>] (reg: &mut $crate::ComponentRegistry) -> $crate::ComponentDesc {
                    fn init_attr(desc:$crate::ComponentDesc) -> Box<[$crate::AttributeEntry]> {
                        let _component: $crate::component2::Component<$ty> = $crate::component2::Component::new(desc);

                        Box::new([
                            $(
                                $(
                                    $crate::AttributeEntry::new::<$attr, $ty, _>(_component, ($($params),*)),
                                )*
                            )*
                        ])
                    }

                    static ATTRS: $crate::OnceCell<Box<[$crate::AttributeEntry]>> = $crate::OnceCell::new();

                    static NAMESPACE: &str = concat!("core::", $ns);

                    static VTABLE: &$crate::ComponentVTable<$ty> = &$crate::ComponentVTable::construct(
                        NAMESPACE,
                        stringify!($name),
                        |desc, key| $crate::component2::slice_attrs(ATTRS.get_or_init(|| init_attr(desc)), key)
                    );

                    *[<comp_ $name>].get_or_init(|| {
                        reg.register2(unsafe { VTABLE.erase() } )
                    })
                }

                $(#[$outer])*
                pub fn $name() -> $crate::component2::Component<$ty> {

                    let desc = *[<comp_ $name>].get()
                        .expect(concat!("Component: ", "core::", $ns, stringify!($name), "is not initialized"));

                    $crate::component2::Component::new(desc)
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

    use serde::de::DeserializeSeed;

    use super::*;
    use crate::ComponentVTable;

    #[test]
    fn manual_component() {
        static VTABLE: &ComponentVTable<String> = &ComponentVTable::construct("core::test", "my_component", |_, _| None);

        let component: Component<String> = Component::new(ComponentDesc::new(1, unsafe { VTABLE.erase() }));

        let value = ComponentEntry::new(component, "Hello, World".into());

        let value2 = value.clone();

        let s = value.downcast_ref::<String>();
        let s2 = value2.downcast_ref::<String>();

        // Since they are cloned, they should not be reference equal
        assert!(!ptr::eq(s as *const String, s2 as *const String));
        // They are however value equal
        assert_eq!(value.downcast_ref::<String>(), value2.downcast_ref::<String>());

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

        assert_eq!(component.name(), "foo");

        assert!(component.attribute::<Serializable>().is_some());

        let p = Person { name: "Adam".into(), age: 28 };
        let entry = ComponentEntry::new(person(), p);

        let str = serde_json::to_string_pretty(entry.attribute::<Serializable>().unwrap().serialize(&entry)).unwrap();

        eprintln!("Serialized: {str}");

        let deserialize = person().attribute::<Serializable>().unwrap();

        let value: ComponentEntry = deserialize.deserialize(&mut serde_json::Deserializer::from_str(&str)).unwrap();

        eprintln!("Value: {:?}", value.as_debug());

        let p = entry.into_inner::<Person>();
        assert_eq!(value.downcast_ref::<Person>(), &p);
        assert_eq!(value.try_downcast_ref::<String>(), None);
    }

    #[test]
    fn make_default() {
        fn default_person() -> Person {
            Person { age: 21, name: "unnamed".into() }
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

        let mut people = people_desc.attribute::<MakeDefault>().unwrap().make_default();

        let mut person = person_desc.attribute::<MakeDefault>().unwrap().make_default();

        assert_eq!(person.downcast_ref::<Person>(), &Person { age: 21, name: "unnamed".into() });

        people.downcast_mut::<Vec<Person>>().push(person.downcast_cloned::<Person>());

        assert_eq!(&people.downcast_mut::<Vec<Person>>()[0], person.downcast_ref::<Person>());
        person.downcast_mut::<Person>().name = "Smith".to_string();
        assert_ne!(&people.downcast_mut::<Vec<Person>>()[0], person.downcast_ref::<Person>());

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

            let value3 = value2.clone();
            let value4: Arc<String> = value2.downcast_cloned();
            assert_eq!(Arc::strong_count(&shared), 4);
            drop(value4);
            assert_eq!(Arc::strong_count(&shared), 3);

            assert_eq!(value3.downcast_ref::<Arc<String>>(), &shared);
            assert_eq!(value3.downcast_ref::<Arc<String>>(), value2.downcast_ref());

            assert!(!ptr::eq(value3.downcast_ref::<Arc<String>>() as *const Arc<String>, &shared as *const _));
            assert!(!ptr::eq(value3.downcast_ref::<Arc<String>>() as *const Arc<String>, value2.downcast_ref::<Arc<String>>() as *const _));

            assert!(ptr::eq(&**value3.downcast_ref::<Arc<String>>() as *const String, &*shared as *const _));
        }

        assert_eq!(Arc::strong_count(&shared), 1);
    }
}
