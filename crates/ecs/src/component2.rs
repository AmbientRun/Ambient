use std::{
    any::{Any, TypeId}, marker::PhantomData, mem::{self, ManuallyDrop}, ptr::NonNull
};

use crate::ComponentValueBase;

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

impl<T> Component<T> {
    fn new(index: i32, vtable: &'static ComponentVTable) -> Self {
        Self { index, vtable, _marker: PhantomData }
    }
}

unsafe fn impl_drop<T: ComponentValue>(value: Box<ComponentHolder<()>>) {
    // Cast the box
    let value: Box<ComponentHolder<T>> = Box::from_raw(Box::into_raw(value).cast::<ComponentHolder<T>>());
    drop(value)
}

unsafe fn impl_clone<T: ComponentValue + Clone>(value: &ComponentHolder<()>) -> Box<ComponentHolder<()>> {
    let value = value.cast::<T>();
    ComponentHolder::construct(value.vtable, value.index, value.object.clone())
}

unsafe fn impl_default<T: ComponentValue + Default>(vtable: &'static ComponentVTable, index: i32) -> Box<ComponentHolder<()>> {
    ComponentHolder::construct(vtable, index, T::default())
}

/// Holds untyped information for everything a component can do
struct ComponentVTable {
    pub component_name: &'static str,
    // TODO: use const value when stabilized
    // https://github.com/rust-lang/rust/issues/63084
    pub get_type_name: fn() -> &'static str,
    pub get_type_id: fn() -> TypeId,

    pub impl_drop: unsafe fn(Box<ComponentHolder<()>>),
    pub impl_clone: unsafe fn(&ComponentHolder<()>) -> Box<ComponentHolder<()>>,
    pub impl_default: Option<unsafe fn(&'static ComponentVTable, i32) -> Box<ComponentHolder<()>>>,

    pub serialize: Option<fn(&dyn ComponentValue) -> &dyn erased_serde::Serialize>,
    pub custom_attrs: fn(&str) -> Option<&'static Attribute>,
}

struct ComponentHolder<T> {
    index: i32,
    vtable: &'static ComponentVTable,
    /// The value.
    ///
    /// **Note**: Do not access manually as the actual `T` type may be different due to type
    /// erasure
    object: T,
}

struct DynComponent {
    inner: ManuallyDrop<Box<ComponentHolder<()>>>,
}

impl DynComponent {
    /// Creates a type erased component
    pub fn new<T: ComponentValue>(component: Component<T>, value: T) -> Self {
        let inner = unsafe { ComponentHolder::construct(component.vtable, component.index, value) };
        let inner = ManuallyDrop::new(inner);

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
        let inner = ManuallyDrop::new(inner);
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
    unsafe fn construct<T: ComponentValue>(vtable: &'static ComponentVTable, index: i32, object: T) -> Box<Self> {
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
        mem::transmute::<Box<ComponentHolder<T>>, Box<ComponentHolder<()>>>(value)
    }
}

impl<T> ComponentHolder<T> {}

#[cfg(test)]
mod test {
    use std::{any::type_name, ptr};

    use itertools::assert_equal;
    use serde::de::value;

    use super::*;

    #[test]
    fn manual_component() {
        static ATTRS: &[Attribute] = &[
            Attribute::new("is_networked", &()),
            Attribute::new("is_stored", &()),
            Attribute::new("display", &|v: &dyn Any| format!("value: {}", v.downcast_ref::<String>().unwrap())),
        ];

        static VTABLE: &ComponentVTable = &ComponentVTable {
            // let vtable: &'static ComponentVTable = &ComponentVTable {
            component_name: "my_component",
            get_type_name: || type_name::<String>(),
            get_type_id: || TypeId::of::<String>(),
            impl_clone: impl_clone::<String>,
            impl_drop: impl_drop::<String>,
            impl_default: Some(impl_default::<String>),
            serialize: None,
            custom_attrs: |key| slice_attrs(ATTRS, key),
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
}
