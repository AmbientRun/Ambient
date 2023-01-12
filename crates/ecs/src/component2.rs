use std::{
    any::TypeId, cmp::Ordering, mem::{self, ManuallyDrop, MaybeUninit}
};

/// Defines an object safe trait which allows for downcasting
pub trait ComponentValue: 'static + Send + Sync {}

impl<T: 'static + Send + Sync> ComponentValue for T {}

pub struct Attribute {
    key: &'static str,
    /// To use: cast to the correct type, which is determined by the attribute in use.
    ///
    /// It is recommended to use helper functions
    value: &'static dyn ComponentValue,
}

impl Attribute {
    pub const fn new(key: &'static str, value: &'static dyn ComponentValue) -> Self {
        Self { key, value }
    }
}

/// Construct attributes from a slice
#[inline(always)]
pub fn slice_attrs(attrs: &'static [Attribute], key: &str) -> Option<&'static Attribute> {
    attrs.iter().find(|v| v.key == key)
}

/// Represents a
pub struct Component<T: 'static> {
    index: i32,
    vtable: &'static ComponentVTable<T>,
}

impl<T> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self { index: self.index, vtable: self.vtable }
    }
}

impl<T> Copy for Component<T> {}

impl<T> PartialEq for Component<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Component<T> {}

impl<T> PartialOrd for Component<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<T> Ord for Component<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Component<T> {
    fn new(index: i32, vtable: &'static ComponentVTable<T>) -> Self {
        Self { index, vtable }
    }
}

fn impl_default<T: ComponentValue + Default>(vtable: &'static ComponentVTable<T>, index: i32) -> ErasedHolder {
    ComponentHolder::construct(vtable, index, T::default())
}

/// Holds untyped information for everything a component can do
#[repr(C)]
struct ComponentVTable<T: 'static> {
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
    impl_default: Option<fn(&'static ComponentVTable<T>, i32) -> ErasedHolder>,
    impl_ser: Option<fn(&ComponentHolder<T>) -> &dyn erased_serde::Serialize>,

    impl_take: fn(Box<ComponentHolder<T>>, dst: *mut MaybeUninit<T>),

    pub serialize: Option<fn(&dyn ComponentValue) -> &dyn erased_serde::Serialize>,
    pub custom_attrs: fn(&str) -> Option<&'static Attribute>,
}

impl<T: Clone + ComponentValue> ComponentVTable<T> {
    /// Creates a new vtable of `T` without any additional bounds
    pub const fn construct(component_name: &'static str) -> Self {
        fn impl_drop<T>(holder: Box<ComponentHolder<T>>) {
            mem::drop(holder)
        }

        fn impl_clone<T: Clone + ComponentValue>(holder: &ComponentHolder<T>) -> ErasedHolder {
            let object = &holder.object;
            ComponentHolder::construct::<T>(holder.vtable, holder.index, T::clone(object))
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
            impl_default: None,
            serialize: None,
            custom_attrs: |_| None,
            impl_ser: None,
        }
    }
}

#[repr(C)]
struct ComponentHolder<T: 'static> {
    index: i32,
    vtable: &'static ComponentVTable<T>,
    /// The value.
    ///
    /// **Note**: Do not access manually as the actual `T` type may be different due to type
    /// erasure
    object: T,
}

impl Drop for DynComponent {
    fn drop(&mut self) {
        unsafe {
            // Drop is only called once.
            // The pointer is safe to read and drop
            // Delegate to the actual drop impl of T
            let inner = ManuallyDrop::take(&mut self.inner);
            let d = (inner).vtable.impl_drop;
            (d)(inner);
        }
    }
}

type ErasedHolder = ManuallyDrop<Box<ComponentHolder<()>>>;

/// Represents a type erased component and value
pub struct DynComponent {
    inner: ErasedHolder,
}

impl DynComponent {
    /// Creates a type erased component
    pub fn new<T: ComponentValue>(component: Component<T>, value: T) -> Self {
        let inner = ComponentHolder::construct(component.vtable, component.index, value);

        Self { inner }
    }

    #[inline]
    /// Returns true if the entry is of type `T`
    pub fn is<T: 'static>(&self) -> bool {
        (self.inner.vtable.get_type_id)() == TypeId::of::<T>()
    }

    /// Attempt to downcast the value to type `T`
    fn try_downcast_ref<T: 'static>(&self) -> Option<&T> {
        if self.is::<T>() {
            // let v = unsafe { self.inner.cast::<T>() };
            unsafe {
                let v = &*(&self.inner.object as *const () as *const T);
                Some(v)
            }
            // Some(&v.object)
        } else {
            None
        }
    }

    fn downcast_ref<T: 'static>(&self) -> &T {
        #[cfg(debug_assertions)]
        match self.try_downcast_ref() {
            Some(v) => v,
            None => panic!("Mismatched types. Expected: {:?}. Found {:?}", (self.inner.vtable.get_type_name)(), std::any::type_name::<T>()),
        }
        #[cfg(not(debug_assertions))]
        self.try_downcast_ref().expect("Mismatched types")
    }

    /// Take a concrete value out of the DynComponent.
    fn take<T: 'static>(self) -> Option<T> {
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
                (inner.vtable.impl_take)(inner, p);
                Some(dst.assume_init())
            }
        } else {
            None
        }
    }
}

impl Clone for DynComponent {
    fn clone(&self) -> Self {
        let inner = (self.inner.vtable.impl_clone)(&self.inner);
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
    fn construct<T: ComponentValue>(vtable: &'static ComponentVTable<T>, index: i32, object: T) -> ErasedHolder {
        debug_assert_eq!(
            (vtable.get_type_id)(),
            TypeId::of::<T>(),
            "Attempt to construct T with an invalid vtable. Expected: {:?}, found: {:?}",
            (vtable.get_type_name)(),
            std::any::type_name::<T>()
        );

        let value = Box::new(ComponentHolder { index, vtable, object });

        // Erase the inner type of the ComponentHolder.
        //
        // This is equivalent to an unsized coercion from Box<ComponentHolder> to
        // Box<ComponentHolder<dyn ComponentValue>>;
        let value = unsafe { mem::transmute::<Box<ComponentHolder<T>>, Box<ComponentHolder<()>>>(value) };
        // Signify that the caller needs to take special care when destructuring this
        ManuallyDrop::new(value)
    }
}

impl<T> ComponentHolder<T> {}

#[cfg(test)]
mod test {
    use std::{ptr, sync::Arc};

    use super::*;

    #[test]
    fn manual_component() {
        static ATTRS: &[Attribute] = &[
            Attribute::new("is_networked", &()),
            Attribute::new("is_stored", &()),
            Attribute::new("display", &|v: &DynComponent| format!("value: {}", v.downcast_ref::<String>())),
        ];

        // let vtable: &'static ComponentVTable = &ComponentVTable {
        static VTABLE: &ComponentVTable<String> = &ComponentVTable {
            impl_default: Some(impl_default::<String>),
            custom_attrs: |key| slice_attrs(ATTRS, key),
            ..ComponentVTable::construct("my_component")
        };

        let component: Component<String> = Component::new(1, VTABLE);

        let value = DynComponent::new(component, "Hello, World".into());

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
    fn test_take() {
        static VTABLE: &ComponentVTable<Arc<String>> = &ComponentVTable::construct("my_component");

        let shared = Arc::new("Foo".to_string());

        let component = Component::new(1, VTABLE);
        {
            let value = DynComponent::new(component, shared.clone());
            let value2 = DynComponent::new(component, shared.clone());

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
        static VTABLE: &ComponentVTable<Arc<String>> = &ComponentVTable::construct("my_component");

        let shared = Arc::new("Foo".to_string());

        let component = Component::new(1, VTABLE);
        {
            let value = DynComponent::new(component, shared.clone());
            let value2 = DynComponent::new(component, shared.clone());

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
