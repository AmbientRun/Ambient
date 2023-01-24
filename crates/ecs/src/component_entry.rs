use std::{
    any::{Any, TypeId}, fmt::Debug, mem::{self, ManuallyDrop, MaybeUninit}
};

use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard};

use crate::{
    component::{ComponentBuffer, IComponentBuffer}, get_external_attributes, get_external_attributes_init, AttributeStore, Component, ComponentDesc, ComponentRegistry, ComponentValue
};

pub(crate) type ErasedHolder = ManuallyDrop<Box<ComponentHolder<()>>>;

pub type AttributeGuard<A> = MappedRwLockReadGuard<'static, A>;
pub type AttributeStoreGuard = MappedRwLockReadGuard<'static, AttributeStore>;
pub type AttributeStoreGuardMut = MappedRwLockWriteGuard<'static, AttributeStore>;

/// Holds untyped information for everything a component can do
#[repr(C)]
pub struct ComponentVTable<T: 'static> {
    pub(crate) path: Option<&'static str>,
    // TODO: use const value when stabilized
    // https://github.com/rust-lang/rust/issues/63084
    pub(crate) get_type_name: fn() -> &'static str,
    pub(crate) get_type_id: fn() -> TypeId,

    pub(crate) impl_create_buffer: fn(ComponentDesc) -> Box<dyn IComponentBuffer>,

    /// # Safety
    /// Drops the inner value
    /// The passed holder must not be used.
    /// See: [`std::ptr::drop_in_place`]
    pub(crate) impl_drop: fn(Box<ComponentHolder<T>>),
    pub(crate) impl_clone: fn(&ComponentHolder<T>) -> ErasedHolder,
    pub(crate) impl_as_any: fn(&ComponentHolder<T>) -> &dyn Any,
    pub(crate) impl_downcast_cloned: fn(&ComponentHolder<T>, dst: *mut MaybeUninit<T>),
    pub(crate) impl_take: fn(Box<ComponentHolder<T>>, dst: *mut MaybeUninit<T>),

    pub(crate) attributes: fn(ComponentDesc) -> AttributeStoreGuard,
    pub(crate) attributes_init: fn(ComponentDesc) -> AttributeStoreGuardMut,
}

impl<T: Clone + ComponentValue> ComponentVTable<T> {
    /// Creates a new vtable of `T` without any additional bounds
    pub const fn construct(
        path: &'static str,
        attributes: fn(ComponentDesc) -> AttributeStoreGuard,
        attributes_init: fn(ComponentDesc) -> AttributeStoreGuardMut,
    ) -> Self {
        Self::construct_inner(Some(path), attributes, attributes_init)
    }

    /// Construct a vtable for a component where the name and attributes are not statically known.
    ///
    /// Must be hydrated by ComponentRegistry
    pub const fn construct_external() -> Self {
        Self::construct_inner(None, |desc| get_external_attributes(desc.index()), |desc| get_external_attributes_init(desc.index()))
    }

    /// Creates a new vtable of `T` without any additional bounds
    const fn construct_inner(
        path: Option<&'static str>,
        attributes: fn(ComponentDesc) -> AttributeStoreGuard,
        attributes_init: fn(ComponentDesc) -> AttributeStoreGuardMut,
    ) -> Self {
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
            // This is safe because `ComponentHolder` does not have a drop impl, so rust's normal
            // drop glue follows, where `object` is skipped
            let v = holder.object;
            unsafe { MaybeUninit::write(&mut *dst, v) };
        }

        fn impl_as_any<T: 'static>(holder: &ComponentHolder<T>) -> &dyn Any {
            &holder.object
        }

        fn impl_create_buffer<T: ComponentValue + Clone>(desc: ComponentDesc) -> Box<dyn IComponentBuffer> {
            Box::new(ComponentBuffer::new(Component::<T>::new(desc)))
        }

        Self {
            path,
            get_type_name: || std::any::type_name::<T>(),
            get_type_id: || TypeId::of::<T>(),
            impl_clone: impl_clone::<T>,
            impl_drop: impl_drop::<T>,
            impl_downcast_cloned: impl_downcast_cloned::<T>,
            impl_take: impl_take::<T>,
            impl_as_any: impl_as_any::<T>,
            impl_create_buffer: impl_create_buffer::<T>,
            attributes,
            attributes_init,
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
    pub const unsafe fn erase(&'static self) -> &'static ComponentVTable<()> {
        mem::transmute::<&'static ComponentVTable<T>, &'static ComponentVTable<()>>(self)
    }
}

#[repr(C)]
pub(crate) struct ComponentHolder<T: 'static> {
    pub(crate) desc: ComponentDesc,
    /// The value.
    ///
    /// **Note**: Do not access manually as the actual `T` type may be different due to type
    /// erasure
    pub(crate) object: T,
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

/// Represents a type erased component and value
pub struct ComponentEntry {
    inner: ErasedHolder,
}

impl std::ops::Deref for ComponentEntry {
    type Target = ComponentDesc;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner.desc
    }
}

impl Debug for ComponentEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.debug_struct("ComponentEntry")
                .field("name", &self.name())
                .field("index", &self.index())
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
    #[inline]
    pub fn desc(&self) -> ComponentDesc {
        self.inner.desc
    }

    /// Creates a type erased component
    pub fn new<T: ComponentValue>(component: Component<T>, value: T) -> Self {
        Self::from_raw_parts(component.desc(), value)
    }

    /// Creates a type erased component
    pub fn from_raw_parts<T: ComponentValue>(desc: ComponentDesc, value: T) -> Self {
        let inner = ComponentHolder::construct(desc, value);

        Self { inner }
    }

    pub fn as_any(&self) -> &dyn Any {
        (self.desc().vtable.impl_as_any)(&self.inner)
    }

    impl_infallible!(&Self, downcast_ref, &T);
    impl_infallible!(&mut Self, downcast_mut, &mut T);
    impl_infallible!(&Self, downcast_cloned, T);
    impl_infallible!(Self, into_inner, T);

    /// Attempt to downcast the value to type `T`
    pub fn try_downcast_ref<T: 'static>(&self) -> Option<&T> {
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
    pub fn try_downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
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
            (self.desc().vtable.impl_downcast_cloned)(&self.inner, &mut dst as *mut MaybeUninit<T> as *mut MaybeUninit<()>);

            Some(unsafe { dst.assume_init() })
        } else {
            None
        }
    }

    pub fn as_debug(&self) -> &dyn Debug {
        self.desc().as_debug(self.as_any())
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
    pub(crate) fn construct<T: 'static + Send + Sync>(desc: ComponentDesc, object: T) -> ErasedHolder {
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
