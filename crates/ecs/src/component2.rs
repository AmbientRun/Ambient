use std::{
    any::{Any, TypeId}, cmp::Ordering, marker::PhantomData, mem::{self, ManuallyDrop, MaybeUninit}
};

use once_cell::sync::Lazy;
use serde::{
    de::{self, Visitor}, Deserialize, Serialize
};

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
        $(#($outer))*
        #[derive(Default, Eq, PartialEq, PartialOrd, Hash, Debug, Clone)]
        pub struct $name {}

        impl $crate::ComponentAttribute for $name {
            type Value = $ty;
        }

)        *
    };
}

pub trait ComponentAttributeValue<T> {
    fn construct() -> Self;
}

/// An attribute with no value
pub struct FlagAttribute;

impl FlagAttribute {
    pub const fn construct_attr<T: ComponentValue>() -> Self {
        Self
    }
}

/// Allow serializing a component entry
pub struct ComponentSerializer {
    ser: fn(&ComponentEntry) -> &dyn erased_serde::Serialize,
    deser: fn(ComponentDesc, &mut dyn erased_serde::Deserializer) -> Result<ComponentEntry, erased_serde::Error>,
}

impl<T: ComponentValue + Serialize + for<'de> Deserialize<'de>> ComponentAttributeValue<T> for ComponentSerializer {
    fn construct() -> Self {
        Self {
            ser: |v| v.downcast_ref::<T>() as &dyn erased_serde::Serialize,
            deser: |desc, deserializer| {
                let deserializer = unsafe { &mut *deserializer };
                let value = T::deserialize(deserializer)?;
                let entry = ComponentEntry::from_raw_parts(desc, value);
                Ok(entry)
            },
        }
    }
}

impl ComponentSerializer {
    /// Serialize a value
    pub fn serialize<'a>(&self, entry: &'a ComponentEntry) -> &'a dyn erased_serde::Serialize {
        (self.ser)(entry)
    }

    /// Deserialize a value into an untyped ComponentEntry
    pub fn deserialize<'a>(&self, desc: ComponentDesc) -> ComponentEntryDeserializer {
        ComponentEntryDeserializer { desc, deser: self.deser }
    }
}

/// Deserializes a single component value into a `ComponentEntry`
pub struct ComponentEntryDeserializer {
    desc: ComponentDesc,
    deser: fn(ComponentDesc, &mut dyn erased_serde::Deserializer) -> Result<ComponentEntry, erased_serde::Error>,
}

impl<'de> serde::de::DeserializeSeed<'de> for ComponentEntryDeserializer {
    type Value = ComponentEntry;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
        let deserializer = &mut deserializer;
        (self.deser)(self.desc, deserializer).map_err(de::Error::custom)
    }
}

component_attributes! {
    Serializable: ComponentSerializer,
    Store: (),
}

impl<T> ComponentAttributeValue<T> for () {
    fn construct() -> Self {}
}

pub struct AttributeEntry {
    /// const fn TypeId::of is not stable
    key: fn() -> TypeId,
    /// To use: cast to the correct type, which is determined by the attribute in use.
    ///
    /// It is recommended to use helper functions
    value: Lazy<Box<dyn Any + Send + Sync>>,
}

impl AttributeEntry {
    pub const fn new<Attr, T>() -> Self
    where
        Attr: ComponentAttribute,
        Attr::Value: ComponentAttributeValue<T>,
        T: 'static,
    {
        Self { key: || TypeId::of::<Attr>(), value: Lazy::new(|| Box::new(Attr::Value::construct) as Box<dyn Any + Send + Sync>) }
    }
}

/// Construct attributes from a slice
#[inline(always)]
pub fn slice_attrs(attrs: &'static [&'static AttributeEntry], key: TypeId) -> Option<&'static AttributeEntry> {
    attrs.iter().find(|v| (v.key)() == key).as_deref()
}

/// Represents a
pub struct Component<T: 'static> {
    desc: ComponentDesc,
    _marker: PhantomData<T>,
}

impl<T: 'static> Component<T> {
    pub fn name(&self) -> &'static str {
        self.desc.vtable.component_name
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
    fn new(desc: ComponentDesc) -> Self {
        Self { desc, _marker: PhantomData }
    }

    pub fn attribute<A: ComponentAttribute>(&self) -> Option<&'static A::Value> {
        self.desc.vtable.attribute::<A>()
    }
}

fn impl_default<T: ComponentValue + Default>(desc: ComponentDesc) -> ErasedHolder {
    ComponentHolder::construct(desc, T::default())
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
    impl_take: fn(Box<ComponentHolder<T>>, dst: *mut MaybeUninit<T>),

    pub custom_attrs: fn(TypeId) -> Option<&'static AttributeEntry>,
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
            impl_take: impl_take::<T>,
            custom_attrs: |_| None,
        }
    }
}

impl<T: 'static> ComponentVTable<T> {
    pub fn attribute<A: ComponentAttribute>(&self) -> Option<&A::Value> {
        let entry = (self.custom_attrs)(TypeId::of::<A>())?;
        let value = entry.value.downcast_ref().expect("Mismatched attribute types");
        Some(value)
    }

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

impl ComponentEntry {
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

    fn downcast_ref<T: 'static>(&self) -> &T {
        #[cfg(debug_assertions)]
        match self.try_downcast_ref() {
            Some(v) => v,
            None => {
                panic!("Mismatched types. Expected: {:?}. Found {:?}", (self.inner.desc.vtable.get_type_name)(), std::any::type_name::<T>())
            }
        }
        #[cfg(not(debug_assertions))]
        self.try_downcast_ref().expect("Mismatched types")
    }

    /// Take a concrete value out of the DynComponent.
    pub fn take<T: 'static>(self) -> Option<T> {
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

    pub fn attribute<A: ComponentAttribute>(&self) -> Option<&'static A::Value> {
        self.inner.desc.vtable.attribute::<A>()
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

    /// Construct a new type erased instance of `T` and return a pointer to the allocation.
    ///
    /// The returned pointer must be deallocated manually
    ///
    /// # Safety
    /// The vtable must be of the type `T`
    ///
    /// T **must** be 'static + Send + Sync to be coerced to `ComponentHolder<()>`
    fn construct<T: ComponentValue>(desc: ComponentDesc, object: T) -> ErasedHolder {
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
    ($($(#[$outer:meta])* $(@[$($attr: ty),*])? $vis: vis $name:ident: $ty:ty,)*) => {
        $(
            $crate::paste::paste! {
                #[allow(non_upper_case_globals)]
                static mut [<comp_ $name>]: i32 = -1;
                $(#[$outer])*
                $vis fn $name() -> Component<$ty> {
                    static ATTRS: &[&$crate::AttributeEntry] = &[
                        $(
                            $(
                                &$crate::AttributeEntry::new::<$attr, $ty>(),
                            )*
                        )*
                    ];

                    static VTABLE: &ComponentVTable<$ty> = &ComponentVTable {
                        custom_attrs: |key| slice_attrs(ATTRS, key),
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

    use super::*;

    #[test]
    fn manual_component() {
        static ATTRS: &[AttributeEntry] = &[];

        // let vtable: &'static ComponentVTable = &ComponentVTable {
        static VTABLE: &ComponentVTable<String> =
            &ComponentVTable { custom_attrs: |key| slice_attrs(ATTRS, key), ..ComponentVTable::construct("my_component") };

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

    #[test]
    fn component_macro() {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct Person {
            name: String,
            age: i32,
        }

        components2! {
            /// Declares a component
            @[Serializable, Store]
            foo: String,
            @[Serializable]
            person: Person,
        }

        let component = foo();

        assert_eq!(component.name(), "foo");

        assert!(component.attribute::<Serializable>().is_some());

        let p = Person { name: "Adam".into(), age: 28 };
        let entry = ComponentEntry::new(person(), p);

        let str = serde_json::to_string_pretty(entry.attribute::<Serializable>().unwrap().serialize(&entry)).unwrap();

        eprintln!("Serialized: {str}");
    }

    #[test]
    fn test_take() {
        components2! {
            my_component: Arc<String>,
        }

        let shared = Arc::new("Foo".to_string());

        {
            let value = ComponentEntry::new(my_component(), shared.clone());
            let value2 = ComponentEntry::new(my_component(), shared.clone());

            assert_eq!(Arc::strong_count(&shared), 3);
            drop(value);
            assert_eq!(Arc::strong_count(&shared), 2);

            let value = value2.take::<Arc<String>>().unwrap();
            assert_eq!(Arc::strong_count(&shared), 2);
            drop(value);
            assert_eq!(Arc::strong_count(&shared), 1);
        }

        assert_eq!(Arc::strong_count(&shared), 1);
    }

    #[test]
    fn leak_test() {
        let shared = Arc::new("Foo".to_string());

        components2! {
            my_component: Arc<String>,
        }

        {
            let value = ComponentEntry::new(my_component(), shared.clone());
            let value2 = ComponentEntry::new(my_component(), shared.clone());

            assert_eq!(Arc::strong_count(&shared), 3);
            drop(value);
            assert_eq!(Arc::strong_count(&shared), 2);

            let value3 = value2.clone();
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
