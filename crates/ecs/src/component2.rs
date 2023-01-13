use std::{
    any::{Any, TypeId}, cmp::Ordering, fmt::Debug, marker::PhantomData, mem::{self, ManuallyDrop, MaybeUninit}
};

use serde::{self, Deserialize, Serialize};

/// Defines an object safe trait which allows for downcasting
pub trait ComponentValue: 'static + Send + Sync {}

impl<T: 'static + Send + Sync> ComponentValue for T {}

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
    debug: fn(&ComponentEntry) -> &dyn Debug,
}

impl<T> ComponentAttributeValue<T, ()> for ComponentDebugger
where
    T: Debug,
{
    fn construct(_: Component<T>, _: ()) -> Self {
        Self { debug: |entry| entry.downcast_ref::<T>() as &dyn Debug }
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

impl<T: 'static> Component<T> {
    pub fn name(&self) -> &'static str {
        self.desc.vtable.component_name
    }

    /// Returns an untyped description of the component key
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
        self.desc.index == other.desc.index
    }
}

impl<T> Eq for Component<T> {}

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

impl<T> Component<T> {
    /// Create a component key from the untyped description
    pub fn new(desc: ComponentDesc) -> Self {
        Self { desc, _marker: PhantomData }
    }

    pub fn attribute<A: ComponentAttribute>(&self) -> Option<&'static A::Value> {
        self.desc.attribute::<A>()
    }
}

/// Holds untyped information for everything a component can do
#[repr(C)]
pub struct ComponentVTable<T: 'static> {
    component_name: &'static str,
    // TODO: use const value when stabilized
    // https://github.com/rust-lang/rust/issues/63084
    get_type_name: fn() -> &'static str,
    get_type_id: fn() -> TypeId,

    /// # Safety
    /// Drops the inner value
    /// The passed holder must not be used.
    /// See: [`std::ptr::drop_in_place`]
    impl_drop: fn(Box<ComponentHolder<T>>),
    impl_clone: fn(&ComponentHolder<T>) -> ErasedHolder,
    impl_downcast_cloned: fn(&ComponentHolder<T>, dst: *mut MaybeUninit<T>),
    impl_take: fn(Box<ComponentHolder<T>>, dst: *mut MaybeUninit<T>),

    pub custom_attrs: fn(ComponentDesc, TypeId) -> Option<&'static AttributeEntry>,
}

impl<T: Clone + ComponentValue> ComponentVTable<T> {
    /// Creates a new vtable of `T` without any additional bounds
    pub const fn construct(component_name: &'static str) -> Self {
        fn impl_drop<T>(holder: Box<ComponentHolder<T>>) {
            mem::drop(holder)
        }

        fn impl_clone<T: Clone + ComponentValue>(holder: &ComponentHolder<T>) -> ErasedHolder {
            let object = &holder.object;
            ComponentHolder::construct::<T>(holder.desc, T::clone(object))
        }

        fn impl_downcast_cloned<T: Clone + ComponentValue>(holder: &ComponentHolder<T>, dst: *mut MaybeUninit<T>) {
            let object = T::clone(&holder.object);
            // Write into the destination
            unsafe {
                MaybeUninit::write(&mut *dst, object);
            }
        }

        #[allow(clippy::boxed_local)]
        fn impl_take<T: ComponentValue>(holder: Box<ComponentHolder<T>>, dst: *mut MaybeUninit<T>) {
            // Take v, but drop the rest
            // This is safe because `ComponentHolder` does not have a drop impl, so rusts normal
            // drop glue follows, where `object` is skipped
            let v = holder.object;
            unsafe { MaybeUninit::write(&mut *dst, v) };
        }

        Self {
            component_name,
            get_type_name: || std::any::type_name::<T>(),
            get_type_id: || TypeId::of::<T>(),
            impl_clone: impl_clone::<T>,
            impl_drop: impl_drop::<T>,
            impl_downcast_cloned: impl_downcast_cloned::<T>,
            impl_take: impl_take::<T>,
            custom_attrs: |_, _| None,
        }
    }
}

impl<T: 'static> ComponentVTable<T> {
    /// Erases the vtable
    ///
    /// # Safety
    /// The table fields **must** have the same layout for all `T`.
    ///
    /// More specifically:
    /// No fields whose size is dependent on `T`.
    /// No fn-ptr fields whose arguments type layout or return value are dependent on self.
    ///
    /// `Box<T>`, `&T`, `*const T`, `*mut T` and similar thin pointers are *safe*.
    ///
    /// `&mut T` is *unsafe* due to pointer variance
    pub unsafe fn erase(&'static self) -> &'static ComponentVTable<()> {
        mem::transmute::<&'static ComponentVTable<T>, &'static ComponentVTable<()>>(self)
    }
}

/// Contains enough information to construct, erase, and de-erase a component key.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComponentDesc {
    index: i32,
    vtable: &'static ComponentVTable<()>,
}

impl ComponentDesc {
    pub fn new(index: i32, vtable: &'static ComponentVTable<()>) -> Self {
        Self { index, vtable }
    }

    pub fn attribute<A: ComponentAttribute>(&self) -> Option<&'static A::Value> {
        let entry = (self.vtable.custom_attrs)(*self, TypeId::of::<A>())?;
        let value = entry.value.downcast_ref().expect("Mismatched attribute types");
        Some(value)
    }
}

#[repr(C)]
struct ComponentHolder<T: 'static> {
    desc: ComponentDesc,
    /// The value.
    ///
    /// **Note**: Do not access manually as the actual `T` type may be different due to type
    /// erasure
    object: T,
}

impl Drop for ComponentEntry {
    fn drop(&mut self) {
        unsafe {
            // Drop is only called once.
            // The pointer is safe to read and drop
            // Delegate to the actual drop impl of T
            let inner = ManuallyDrop::take(&mut self.inner);
            let d = (inner).desc.vtable.impl_drop;
            (d)(inner);
        }
    }
}

type ErasedHolder = ManuallyDrop<Box<ComponentHolder<()>>>;

/// Represents a type erased component and value
pub struct ComponentEntry {
    inner: ErasedHolder,
}

impl Debug for ComponentEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.debug_struct("ComponentEntry")
                .field("name", &self.name())
                .field("index", &self.inner.desc.index)
                .field("value", self.as_debug())
                .finish()
        } else {
            self.as_debug().fmt(f)
        }
    }
}

macro_rules! impl_infallible {
    ($recv: ty, $name: ident, $ret: ty) => {
        paste::paste! {
            #[doc = "See [`" try_ $name "`]" ]
            #[doc = "# Panics"]
            #[doc = "If the types do not match"]
            pub fn $name<T: 'static>(self: $recv) -> $ret {
                #[cfg(debug_assertions)]
                {
                    let ty = self.type_name();
                    match self.[< try_ $name >]() {
                        Some(v) => v,
                        None => {
                            panic!(
                                "Mismatched types. Attempt to downcast {:?} into {:?}",
                                ty,
                                std::any::type_name::<T>()
                            )
                        }
                    }
                }
                #[cfg(not(debug_assertions))]
                self.[< try_ $name>]().expect("Mismatched types")
            }
        }
    };
}

impl ComponentEntry {
    pub fn name(&self) -> &'static str {
        self.inner.desc.vtable.component_name
    }

    pub fn type_name(&self) -> &'static str {
        (self.inner.desc.vtable.get_type_name)()
    }

    /// Creates a type erased component
    pub fn new<T: ComponentValue>(component: Component<T>, value: T) -> Self {
        Self::from_raw_parts(component.desc, value)
    }

    /// Creates a type erased component
    fn from_raw_parts<T: ComponentValue>(desc: ComponentDesc, value: T) -> Self {
        let inner = ComponentHolder::construct(desc, value);

        Self { inner }
    }

    #[inline]
    /// Returns true if the entry is of type `T`
    pub fn is<T: 'static>(&self) -> bool {
        (self.inner.desc.vtable.get_type_id)() == TypeId::of::<T>()
    }

    impl_infallible!(&Self, downcast_ref, &T);
    impl_infallible!(&mut Self, downcast_mut, &mut T);
    impl_infallible!(&Self, downcast_cloned, T);
    impl_infallible!(Self, into_inner, T);

    /// Attempt to downcast the value to type `T`
    fn try_downcast_ref<T: 'static>(&self) -> Option<&T> {
        if self.is::<T>() {
            // unsafe {
            //     let v = &*(&self.inner.object as *const () as *const T);
            //     Some(v)
            // }
            let v = unsafe { self.inner.cast::<T>() };
            Some(&v.object)
        } else {
            None
        }
    }

    /// Attempt to downcast the value to type `T`
    fn try_downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            let v = unsafe { self.inner.cast_mut::<T>() };
            Some(&mut v.object)
        } else {
            None
        }
    }

    /// Take a concrete value out of the DynComponent.
    pub fn try_into_inner<T: 'static>(self) -> Option<T> {
        if self.is::<T>() {
            let mut dst = MaybeUninit::uninit();
            // Safety
            // Type is guaranteed
            unsafe {
                // Prevent the Self drop impl from running, so that we can take inner
                // However, we can't just `mem::forget` is as the Box still needs to be
                // deallocated, which is take care of in `impl_take`.
                let mut this = ManuallyDrop::new(self);
                let inner = ManuallyDrop::take(&mut this.inner);
                let p = &mut dst as *mut MaybeUninit<T> as *mut MaybeUninit<()>;
                (inner.desc.vtable.impl_take)(inner, p);
                Some(dst.assume_init())
            }
        } else {
            None
        }
    }

    pub fn try_downcast_cloned<T: 'static>(&self) -> Option<T> {
        if self.is::<T>() {
            let mut dst = MaybeUninit::uninit();
            (self.inner.desc.vtable.impl_downcast_cloned)(&self.inner, &mut dst as *mut MaybeUninit<T> as *mut MaybeUninit<()>);

            Some(unsafe { dst.assume_init() })
        } else {
            None
        }
    }

    pub fn attribute<A: ComponentAttribute>(&self) -> Option<&'static A::Value> {
        self.inner.desc.attribute::<A>()
    }

    pub fn as_debug(&self) -> &dyn Debug {
        struct NoDebug;
        impl Debug for NoDebug {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("...")
            }
        }

        if let Some(v) = self.attribute::<Debuggable>() {
            (v.debug)(self)
        } else {
            &NoDebug
        }
    }
}

impl Clone for ComponentEntry {
    fn clone(&self) -> Self {
        let inner = (self.inner.desc.vtable.impl_clone)(&self.inner);
        Self { inner }
    }
}

impl ComponentHolder<()> {
    unsafe fn cast<T>(&self) -> &ComponentHolder<T> {
        &*(self as *const ComponentHolder<()> as *const ComponentHolder<T>)
    }

    unsafe fn cast_mut<T>(&mut self) -> &mut ComponentHolder<T> {
        &mut *(self as *mut ComponentHolder<()> as *mut ComponentHolder<T>)
    }

    /// Construct a new type erased instance of `T` and return a pointer to the allocation.
    ///
    /// The returned pointer must be deallocated manually
    ///
    /// # Safety
    /// The vtable must be of the type `T`
    ///
    /// T **must** be 'static + Send + Sync to be coerced to `ComponentHolder<()>`
    fn construct<T: 'static + Send + Sync>(desc: ComponentDesc, object: T) -> ErasedHolder {
        debug_assert_eq!(
            (desc.vtable.get_type_id)(),
            TypeId::of::<T>(),
            "Attempt to construct T with an invalid vtable. Expected: {:?}, found: {:?}",
            (desc.vtable.get_type_name)(),
            std::any::type_name::<T>()
        );

        let value = Box::new(ComponentHolder { desc, object });

        // Erase the inner type of the ComponentHolder.
        //
        // This is equivalent to an unsized coercion from Box<ComponentHolder> to
        // Box<ComponentHolder<dyn ComponentValue>>;
        let value = unsafe { mem::transmute::<Box<ComponentHolder<T>>, Box<ComponentHolder<()>>>(value) };
        // Signify that the caller needs to take special care when destructuring this
        ManuallyDrop::new(value)
    }
}

#[macro_export]
macro_rules! components2 {
    ($ns: literal, { $($(#[$outer:meta])* $(@[$($attr: ty$([ $params: tt ])?),*])? $vis: vis $name:ident: $ty:ty,)*}) => {
        $(
            $crate::paste::paste! {
                #[allow(non_upper_case_globals)]
                static mut [<comp_ $name>]: i32 = -1;
                $(#[$outer])*
                $vis fn $name() -> Component<$ty> {
                    fn init_attr(desc:ComponentDesc) -> Box<[AttributeEntry]> {
                        let _component: Component<$ty> = Component::new(desc);
                        Box::new([
                            $(
                                $(
                                    $crate::AttributeEntry::new::<$attr, $ty, _>(_component, ($($params),*)),
                                )*
                            )*
                        ])
                    }

                    static ATTRS: once_cell::sync::OnceCell<Box<[AttributeEntry]>> = once_cell::sync::OnceCell::new();

                    static VTABLE: &ComponentVTable<$ty> = &ComponentVTable {
                        custom_attrs: |desc, key| slice_attrs(ATTRS.get_or_init(|| init_attr(desc)), key),
                        ..ComponentVTable::construct(stringify!($name))
                    };

                    let index = unsafe { [<comp_ $name>] };
                    Component::new(ComponentDesc::new(index, unsafe { VTABLE.erase() }))
                }
            }
        )*
    }
}

#[cfg(test)]
mod test {
    use std::{ptr, sync::Arc};

    use serde::de::DeserializeSeed;

    use super::*;

    #[test]
    fn manual_component() {
        static VTABLE: &ComponentVTable<String> = &ComponentVTable::construct("my_component");

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
    struct Person {
        name: String,
        age: i32,
    }

    #[test]
    fn component_macro() {
        components2! ("test",{
            @[Serializable, Debuggable]
            foo: String,
            /// This is a person
            @[Serializable, Debuggable]
            person: Person,
        });

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

        components2! ("test", {
            @[MakeDefault, Debuggable]
            people: Vec<Person>,
            @[MakeDefault[default_person], Debuggable, Store, Networked]
            person: Person,
        });

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
        components2! ("test", {
            @[Store]
            my_component: Arc<String>,
        });

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

        components2! ("test", {
            my_component: Arc<String>,
        });

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
