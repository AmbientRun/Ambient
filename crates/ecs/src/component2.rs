use std::{
    any::{type_name, Any, TypeId}, cmp::Ordering, marker::PhantomData, mem::{self, ManuallyDrop}, ptr::{self, NonNull}
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

pub fn slice_attrs(attrs: &'static [Attribute], key: &str) -> Option<&'static Attribute> {
    attrs.iter().find(|v| v.key == key)
}

pub struct Component<T> {
    index: i32,
    vtable: &'static ComponentVTable,
    _marker: PhantomData<T>,
}

impl<T> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self { index: self.index, vtable: self.vtable, _marker: PhantomData }
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
    fn new(index: i32, vtable: &'static ComponentVTable) -> Self {
        Self { index, vtable, _marker: PhantomData }
    }
}

unsafe fn impl_clone<T: ComponentValue + Clone>(value: &ComponentHolder<()>) -> ErasedHolder {
    let value = value.cast::<T>();
    ComponentHolder::construct::<T>(value.vtable, value.index, (*value.object).clone())
}

unsafe fn impl_default<T: ComponentValue + Default>(vtable: &'static ComponentVTable, index: i32) -> ErasedHolder {
    ComponentHolder::construct(vtable, index, T::default())
}

/// Holds untyped information for everything a component can do
struct ComponentVTable {
    component_name: &'static str,
    // TODO: use const value when stabilized
    // https://github.com/rust-lang/rust/issues/63084
    get_type_name: fn() -> &'static str,
    get_type_id: fn() -> TypeId,

    /// # Safety
    /// Drops the inner value
    /// The passed holder must not be used.
    /// See: [`std::ptr::drop_in_place`]
    impl_drop: unsafe fn(*mut ManuallyDrop<()>),
    impl_clone: unsafe fn(&ComponentHolder<()>) -> ErasedHolder,
    impl_default: Option<unsafe fn(&'static ComponentVTable, i32) -> ErasedHolder>,

    impl_take: unsafe fn(*mut ComponentHolder<()>),

    pub serialize: Option<fn(&dyn ComponentValue) -> &dyn erased_serde::Serialize>,
    pub custom_attrs: fn(&str) -> Option<&'static Attribute>,
}

impl ComponentVTable {
    /// Creates a new vtable of `T` without any additional bounds
    pub const fn construct<T: ComponentValue + Clone>(component_name: &'static str) -> ComponentVTable {
        unsafe fn drop<T>(object: *mut ManuallyDrop<()>) {
            let object = &mut *(object as *mut ManuallyDrop<T>);
            ManuallyDrop::drop(object);
        }

        Self {
            component_name,
            get_type_name: || std::any::type_name::<T>(),
            get_type_id: || TypeId::of::<T>(),
            impl_clone: impl_clone::<T>,
            impl_drop: drop::<T>,
            impl_default: None,
            serialize: None,
            custom_attrs: |_| None,
            impl_take: |v| {},
        }
    }
}

#[repr(C)]
struct ComponentHolder<T> {
    index: i32,
    vtable: &'static ComponentVTable,
    /// The value.
    ///
    /// **Note**: Do not access manually as the actual `T` type may be different due to type
    /// erasure
    object: ManuallyDrop<T>,
}

type ErasedHolder = Box<ComponentHolder<()>>;

/// Represents a type erased component and value
pub struct DynComponent {
    inner: Box<ComponentHolder<()>>,
}

// Note: as Drop can't be specialized, the `T` is irrelevant as the vtable drop impl will be called
impl<T> Drop for ComponentHolder<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop is only called once.
            // The pointer is safe to read and drop
            // Delegate to the actual drop impl of T
            let erased = &mut *(self as *mut ComponentHolder<T> as *mut ComponentHolder<()>);
            (self.vtable.impl_drop)(&mut erased.object as *mut _);
        }
    }
}

impl DynComponent {
    /// Creates a type erased component
    pub fn new<T: ComponentValue>(component: Component<T>, value: T) -> Self {
        let inner = unsafe { ComponentHolder::construct(component.vtable, component.index, value) };

        Self { inner }
    }

    fn try_downcast_ref<T: 'static>(&self) -> Option<&T> {
        if (self.inner.vtable.get_type_id)() == TypeId::of::<T>() {
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
            None => panic!("Mismatched type. Expected: {:?}. Found {:?}", (self.inner.vtable.get_type_name)(), std::any::type_name::<T>()),
        }
    }
}

impl Clone for DynComponent {
    fn clone(&self) -> Self {
        let inner = unsafe { (self.inner.vtable.impl_clone)(&self.inner) };
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
    /// T **must** be Send+Sync+'static to be coerced to `ComponentHolder<()>`
    unsafe fn construct<T: ComponentValue>(vtable: &'static ComponentVTable, index: i32, object: T) -> ErasedHolder {
        debug_assert_eq!(
            (vtable.get_type_id)(),
            TypeId::of::<T>(),
            "Attempt to construct T with an invalid vtable. Expected: {:?}, found: {:?}",
            (vtable.get_type_name)(),
            std::any::type_name::<T>()
        );

        let value = Box::new(ComponentHolder { index, vtable, object: ManuallyDrop::new(object) });

        // Erase the inner type of the ComponentHolder.
        //
        // This is equivalent to an unsized coercion from Box<ComponentHolder> to
        // Box<ComponentHolder<dyn ComponentValue>>;
        mem::transmute::<Box<ComponentHolder<T>>, Box<ComponentHolder<()>>>(value)
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
            Attribute::new("display", &|v: &dyn Any| format!("value: {}", v.downcast_ref::<String>().unwrap())),
        ];

        // let vtable: &'static ComponentVTable = &ComponentVTable {
        static VTABLE: &ComponentVTable = &ComponentVTable {
            impl_default: Some(impl_default::<String>),
            custom_attrs: |key| slice_attrs(ATTRS, key),
            ..ComponentVTable::construct::<String>("my_component")
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
    fn leak_test() {
        static VTABLE: &ComponentVTable = &ComponentVTable::construct::<Arc<String>>("my_component");

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
