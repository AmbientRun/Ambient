use crate::{
    event, host,
    internal::{
        conversion::{FromBindgen, IntoBindgen},
        runtime::on_async,
        ObjectRef,
    },
    on, EntityId, EntityUid, EventOk, Mat4, Quat, Vec2, Vec3, Vec4,
};

use once_cell::sync::Lazy;
use std::{collections::HashMap, future::Future, marker::PhantomData};

mod internal;

/// Implemented by all [Component]s.
pub trait IComponent {
    #[doc(hidden)]
    fn index(&self) -> u32;
}

/// A component (piece of entity data). See [crate::entity::get_component] and [crate::entity::set_component].
#[derive(Debug)]
pub struct Component<T> {
    index: u32,
    _phantom: PhantomData<T>,
}
impl<T> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _phantom: PhantomData,
        }
    }
}
impl<T> Copy for Component<T> {}
impl<T> Component<T> {
    #[doc(hidden)]
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }
}
impl<T> IComponent for Component<T> {
    fn index(&self) -> u32 {
        self.index
    }
}

#[doc(hidden)]
pub fn internal_get_component<T>(id: &str) -> Component<T> {
    Component::new(host::component_get_index(id).unwrap())
}

#[doc(hidden)]
pub type LazyComponent<T> = Lazy<Component<T>>;
#[doc(hidden)]
#[macro_export]
macro_rules! lazy_component {
    ($id:literal) => {
        $crate::LazyComponent::new(|| $crate::internal_get_component($id))
    };
}

#[doc(hidden)]
pub trait ToParam {
    fn as_param(&self) -> host::ComponentTypeParam<'_>;
}

/// Implemented by all types you can use with [crate::entity::get_component].
pub trait SupportedComponentTypeGet: Send + Sync
where
    Self: Sized,
{
    #[doc(hidden)]
    fn from_result(result: host::ComponentTypeResult) -> Option<Self>;
}

/// Implemented by all types you can use with [crate::entity::set_component].
pub trait SupportedComponentTypeSet: Send + Sync
where
    Self: Sized,
{
    #[doc(hidden)]
    type OwnedParam: ToParam;

    #[doc(hidden)]
    fn into_result(self) -> host::ComponentTypeResult;

    #[doc(hidden)]
    fn into_owned_param(self) -> Self::OwnedParam;
}

macro_rules! define_component_types {
    ($(($type:ty, $value:ident)),*) => {
        $(
        impl SupportedComponentTypeGet for $type {
            fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
                match result {
                    host::ComponentTypeResult::$value(v) => Some(v.from_bindgen()),
                    _ => None,
                }
            }
        }

        impl SupportedComponentTypeSet for $type {
            type OwnedParam = Self;

            fn into_result(self) -> host::ComponentTypeResult {
                host::ComponentTypeResult::$value(self.into_bindgen())
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self
            }
        }

        impl ToParam for $type {
            fn as_param<'a>(&'a self) -> host::ComponentTypeParam<'a> {
                host::ComponentTypeParam::$value((*self).into_bindgen())
            }
        }
        ) *
    };
}

define_component_types!(
    ((), TypeEmpty),
    (bool, TypeBool),
    (EntityId, TypeEntityId),
    (f32, TypeF32),
    (f64, TypeF64),
    (Mat4, TypeMat4),
    (i32, TypeI32),
    (Quat, TypeQuat),
    (u32, TypeU32),
    (u64, TypeU64),
    (Vec2, TypeVec2),
    (Vec3, TypeVec3),
    (Vec4, TypeVec4)
);

impl SupportedComponentTypeGet for ObjectRef {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeObjectRef(v) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl SupportedComponentTypeSet for ObjectRef {
    type OwnedParam = Self;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeObjectRef(self.into_bindgen())
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self
    }
}
impl ToParam for ObjectRef {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeObjectRef(self.into_bindgen())
    }
}

impl SupportedComponentTypeGet for EntityUid {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeEntityUid(v) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl SupportedComponentTypeSet for EntityUid {
    type OwnedParam = Self;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeEntityUid(self.into_bindgen())
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self
    }
}
impl ToParam for EntityUid {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeEntityUid(self.into_bindgen())
    }
}

impl SupportedComponentTypeGet for String {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeString(v) => Some(v),
            _ => None,
        }
    }
}
impl SupportedComponentTypeSet for String {
    type OwnedParam = Self;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeString(self.into_bindgen())
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self
    }
}
impl ToParam for String {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeString(self.as_str())
    }
}

macro_rules! define_vec_opt_component_types {
    ($(($type:ty, $value:ident)),*) => {
        $(
        impl SupportedComponentTypeGet for Vec<$type> {
            fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
                match result {
                    host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::$value(v)) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
                    _ => None,
                }
            }
        }
        impl SupportedComponentTypeSet for Vec<$type> {
            type OwnedParam = Vec<<$type as IntoBindgen>::Item>;

            fn into_result(self) -> host::ComponentTypeResult {
                host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::$value(self.into_bindgen()))
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self.iter().map(|v| (*v).into_bindgen()).collect()
            }
        }
        impl ToParam for Vec<<$type as IntoBindgen>::Item> {
            fn as_param(&self) -> host::ComponentTypeParam<'_> {
                host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::$value(&self))
            }
        }

        impl SupportedComponentTypeGet for Option<$type> {
            fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
                match result {
                    host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::$value(v)) => Some(v.from_bindgen()),
                    _ => None,
                }
            }
        }
        impl SupportedComponentTypeSet for Option<$type> {
            type OwnedParam = Option<<$type as IntoBindgen>::Item>;

            fn into_result(self) -> host::ComponentTypeResult {
                host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::$value(self.into_bindgen()))
            }

            fn into_owned_param(self) -> Self::OwnedParam {
                self.into_bindgen()
            }
        }
        impl ToParam for Option<<$type as IntoBindgen>::Item> {
            fn as_param(&self) -> host::ComponentTypeParam<'_> {
                host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::$value(self.clone()))
            }
        }
        ) *
    };
}

define_vec_opt_component_types!(
    ((), TypeEmpty),
    (bool, TypeBool),
    (EntityId, TypeEntityId),
    (f32, TypeF32),
    (f64, TypeF64),
    (Mat4, TypeMat4),
    (i32, TypeI32),
    (Quat, TypeQuat),
    (u32, TypeU32),
    (u64, TypeU64),
    (Vec2, TypeVec2),
    (Vec3, TypeVec3),
    (Vec4, TypeVec4)
);

impl SupportedComponentTypeGet for Vec<ObjectRef> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeObjectRef(
                v,
            )) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Vec<ObjectRef> {
    type OwnedParam = Vec<host::ObjectRefParam<'a>>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeObjectRef(
            self.iter().map(|s| s.clone().into_bindgen()).collect(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.iter()
            .map(|v| host::ObjectRefParam { id: v.as_ref() })
            .collect()
    }
}
impl<'a> ToParam for Vec<host::ObjectRefParam<'a>> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::TypeObjectRef(self))
    }
}

impl SupportedComponentTypeGet for Option<ObjectRef> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeOption(
                host::ComponentOptionTypeResult::TypeObjectRef(v),
            ) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Option<ObjectRef> {
    type OwnedParam = Option<host::ObjectRefParam<'a>>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::TypeObjectRef(
            self.clone().into_bindgen(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.as_ref()
            .map(|s| host::ObjectRefParam { id: s.as_ref() })
    }
}
impl<'a> ToParam for Option<host::ObjectRefParam<'a>> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::TypeObjectRef(
            self.clone(),
        ))
    }
}

impl SupportedComponentTypeGet for Vec<EntityUid> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeEntityUid(
                v,
            )) => Some(v.into_iter().map(|v| v.from_bindgen()).collect()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Vec<EntityUid> {
    type OwnedParam = Vec<host::EntityUidParam<'a>>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeEntityUid(
            self.iter().map(|s| s.clone().into_bindgen()).collect(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.iter()
            .map(|v| host::EntityUidParam { id: v.as_ref() })
            .collect()
    }
}
impl<'a> ToParam for Vec<host::EntityUidParam<'a>> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::TypeEntityUid(self))
    }
}

impl SupportedComponentTypeGet for Option<EntityUid> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeOption(
                host::ComponentOptionTypeResult::TypeEntityUid(v),
            ) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Option<EntityUid> {
    type OwnedParam = Option<host::EntityUidParam<'a>>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::TypeEntityUid(
            self.clone().into_bindgen(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.as_ref()
            .map(|s| host::EntityUidParam { id: s.as_ref() })
    }
}
impl<'a> ToParam for Option<host::EntityUidParam<'a>> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::TypeEntityUid(
            self.clone(),
        ))
    }
}

impl SupportedComponentTypeGet for Vec<String> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeString(v)) => {
                Some(v.into_iter().map(|v| v.from_bindgen()).collect())
            }
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Vec<String> {
    type OwnedParam = Vec<&'a str>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeList(host::ComponentListTypeResult::TypeString(
            self.iter().map(|s| s.clone().into_bindgen()).collect(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.iter().map(|v| v.as_str()).collect()
    }
}
impl<'a> ToParam for Vec<&'a str> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeList(host::ComponentListTypeParam::TypeString(self))
    }
}

impl SupportedComponentTypeGet for Option<String> {
    fn from_result(result: host::ComponentTypeResult) -> Option<Self> {
        match result {
            host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::TypeString(
                v,
            )) => Some(v.from_bindgen()),
            _ => None,
        }
    }
}
impl<'a> SupportedComponentTypeSet for &'a Option<String> {
    type OwnedParam = Option<&'a str>;

    fn into_result(self) -> host::ComponentTypeResult {
        host::ComponentTypeResult::TypeOption(host::ComponentOptionTypeResult::TypeString(
            self.clone().into_bindgen(),
        ))
    }

    fn into_owned_param(self) -> Self::OwnedParam {
        self.as_ref().map(|s| s.as_str())
    }
}
impl<'a> ToParam for Option<&'a str> {
    fn as_param(&self) -> host::ComponentTypeParam<'_> {
        host::ComponentTypeParam::TypeOption(host::ComponentOptionTypeParam::TypeString(*self))
    }
}

/// Contains event data
#[derive(Clone)]
pub struct Components(pub(crate) HashMap<u32, crate::host::ComponentTypeResult>);
impl Components {
    /// Creates a new `Components`
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Gets component data if it exists
    pub fn get<T: SupportedComponentTypeGet>(&self, component: Component<T>) -> Option<T> {
        T::from_result(self.0.get(&component.index())?.clone())
    }

    /// Set data for `component`
    pub fn set<T: SupportedComponentTypeSet>(&mut self, component: Component<T>, value: T) {
        self.0.insert(component.index(), value.into_result());
    }

    /// Sets data for `component` to the default value for `T`
    pub fn set_default<T: SupportedComponentTypeSet + Default>(&mut self, component: Component<T>) {
        self.set(component, T::default())
    }

    /// Adds `value` to this `EntityData` and returns `self` to allow for easy chaining
    pub fn with<T: SupportedComponentTypeSet>(mut self, component: Component<T>, value: T) -> Self {
        self.set(component, value);
        self
    }

    /// Adds the default value for `T` to this `EntityData` and returns `self` to allow for easy chaining
    pub fn with_default<T: SupportedComponentTypeSet + Default>(
        mut self,
        component: Component<T>,
    ) -> Self {
        self.set_default(component);
        self
    }

    /// Spawns an entity with these components. If `persistent` is set, this entity will not be
    /// removed when this script is unloaded.
    ///
    /// This is an asynchronous operation; use [crate::entity::wait_for_spawn] to get notified when
    /// the entity is spawned.
    ///
    /// Returns `spawned_entity_uid`.
    pub fn spawn(&self, persistent: bool) -> EntityUid {
        crate::entity::spawn(self, persistent)
    }

    pub(crate) fn call_with<R>(
        &self,
        callback: impl FnOnce(&[(u32, host::ComponentTypeParam<'_>)]) -> R,
    ) -> R {
        let data = internal::create_owned_types(&self.0);
        let data = internal::create_borrowed_types(&data);
        callback(&data)
    }
}

/// A tuple of [Component]s. Used with [GeneralQuery], [EventQuery] and [ChangeQuery].
pub trait ComponentsTuple: Send + Sync {
    /// The types of the data stored in this tuple
    type Data;

    #[doc(hidden)]
    fn as_indices(&self) -> Vec<u32>;
    #[doc(hidden)]
    fn from_component_types(component_types: Vec<host::ComponentTypeResult>) -> Option<Self::Data>;
}

// From: https://stackoverflow.com/questions/56697029/is-there-a-way-to-impl-trait-for-a-tuple-that-may-have-any-number-elements
macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: SupportedComponentTypeGet),+> ComponentsTuple for ($(Component<$name>,)+) {
            #[allow(unused_parens)]
            type Data = ($($name),+);

            fn as_indices(&self) -> Vec<u32> {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                vec![$($name.index(),)*]
            }
            fn from_component_types(component_types: Vec<host::ComponentTypeResult>) -> Option<Self::Data> {
                paste::paste! {
                    #[allow(non_snake_case)]
                    if let [$([<value_ $name>],)+] = &component_types[..] {
                        Some(($($name::from_result([<value_ $name>].clone())?),+))
                    } else {
                        None
                    }
                }
            }
        }
    };
}
tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }
impl<T: SupportedComponentTypeGet> ComponentsTuple for Component<T> {
    type Data = T;

    fn as_indices(&self) -> Vec<u32> {
        vec![self.index()]
    }
    fn from_component_types(component_types: Vec<host::ComponentTypeResult>) -> Option<Self::Data> {
        assert_eq!(component_types.len(), 1);
        T::from_result(component_types[0].clone())
    }
}
impl ComponentsTuple for () {
    type Data = ();

    fn as_indices(&self) -> Vec<u32> {
        vec![]
    }
    fn from_component_types(component_types: Vec<host::ComponentTypeResult>) -> Option<Self::Data> {
        assert!(component_types.is_empty());
        Some(())
    }
}

/// Creates a new [GeneralQueryBuilder] that will find entities that have the specified `components`
/// and can be [built|`GeneralQueryBuilder::build`] to create a [GeneralQuery].
///
/// Building a query is somewhat expensive, but they are cheap to copy and evaluate, so it's
/// recommended that you build your queries once and reuse them elsewhere.
pub fn query<Components: ComponentsTuple + Copy + Clone + 'static>(
    components: Components,
) -> GeneralQueryBuilder<Components> {
    GeneralQuery::create(components)
}

/// Creates a new [ChangeQuery] that will find entities that have the specified `components`
/// that will call its bound function when components marked by [ChangeQuery::track_change]
/// change.
pub fn change_query<Components: ComponentsTuple + Copy + Clone + 'static>(
    components: Components,
) -> ChangeQuery<Components> {
    ChangeQuery::create(components)
}

/// Creates a new [EventQuery] that will find entities that have the specified `components`
/// that will call its bound function when an entity with those components are spawned / seen
/// for the first time by this query.
pub fn spawn_query<Components: ComponentsTuple + Copy + Clone + 'static>(
    components: Components,
) -> EventQuery<Components> {
    EventQuery::create(QueryEvent::Spawn, components)
}

/// Creates a new [EventQuery] that will find entities that have the specified `components`
/// that will call its bound function when an entity with those components are despawned / seen
/// for the last time by this query.
pub fn despawn_query<Components: ComponentsTuple + Copy + Clone + 'static>(
    components: Components,
) -> EventQuery<Components> {
    EventQuery::create(QueryEvent::Despawn, components)
}

/// When this [EventQuery] should return results.
pub enum QueryEvent {
    /// When this collection of components is spawned
    Spawn,
    /// When this collection of components is despawned
    Despawn,
}

#[derive(Clone, Copy)]
/// An ECS query used to find entities in the world.
pub struct GeneralQuery<Components: ComponentsTuple + Copy + Clone + 'static>(
    QueryImpl<Components>,
);
impl<Components: ComponentsTuple + Copy + Clone + 'static> GeneralQuery<Components> {
    /// Creates a new [GeneralQueryBuilder] that will find entities that have the specified `components`
    /// and can be [built|`GeneralQueryBuilder::build`] to create a [GeneralQuery].
    ///
    /// Building a query is somewhat expensive, but they are cheap to copy and evaluate, so it's
    /// recommended that you build your queries once and reuse them elsewhere.
    pub fn create(components: Components) -> GeneralQueryBuilder<Components> {
        GeneralQueryBuilder(QueryBuilderImpl::new(components.as_indices()))
    }

    /// Evaluate the query and return the results.
    pub fn evaluate(&self) -> Vec<(EntityId, Components::Data)> {
        self.0.evaluate()
    }

    /// Consume this query and call `callback` (`fn`) each frame with the result of the query.
    pub fn bind(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) + Send + Sync + 'static,
    ) {
        self.0.bind(callback)
    }

    /// Consume this query and call `callback` (`async fn`) each frame with the result of the query.
    pub fn bind_async<R: Future<Output = ()>>(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) -> R + Copy + Send + Sync + 'static,
    ) {
        self.0.bind_async(callback)
    }
}
/// Build a [GeneralQuery] for the ECS. This is how you find entities in the game world.
pub struct GeneralQueryBuilder<Components: ComponentsTuple + Copy + Clone + 'static>(
    QueryBuilderImpl<Components>,
);
impl<Components: ComponentsTuple + Copy + Clone + 'static> GeneralQueryBuilder<Components> {
    /// The entities must include the components in `requires`.
    pub fn requires(mut self, requires: impl ComponentsTuple) -> Self {
        self.0.requires(requires);
        self
    }

    /// The entities must not include the components in `exclude`.
    pub fn excludes(mut self, excludes: impl ComponentsTuple) -> Self {
        self.0.excludes(excludes);
        self
    }

    /// Builds a [GeneralQuery].
    pub fn build(self) -> GeneralQuery<Components> {
        GeneralQuery(QueryImpl::new(
            self.0.build_impl(&[], host::QueryEvent::Frame),
        ))
    }
}

/// An ECS query that calls a callback when entities containing components
/// marked with [ChangeQuery::track_change] have those components change.
pub struct ChangeQuery<Components: ComponentsTuple + Copy + Clone + 'static>(
    QueryBuilderImpl<Components>,
    Vec<u32>,
);
impl<Components: ComponentsTuple + Copy + Clone + 'static> ChangeQuery<Components> {
    /// Creates a new [ChangeQuery] that will find entities that have the specified `components`
    /// that will call its bound function when components marked by [Self::track_change]
    /// change.
    pub fn create(components: Components) -> Self {
        Self(QueryBuilderImpl::new(components.as_indices()), vec![])
    }

    /// The entities must include the components in `requires`.
    pub fn requires(mut self, requires: impl ComponentsTuple) -> Self {
        self.0.requires(requires);
        self
    }

    /// The entities must not include the components in `exclude`.
    pub fn excludes(mut self, excludes: impl ComponentsTuple) -> Self {
        self.0.excludes(excludes);
        self
    }

    /// The query will return results when these components change values.
    ///
    /// Note that this does *not* implicitly [requires] the components; this allows you to track
    /// changes for entities that do not have all of the tracked components.
    pub fn track_change(mut self, changes: impl ComponentsTuple) -> Self {
        self.1.extend_from_slice(&changes.as_indices());
        self
    }

    /// Each time the components marked by [Self::track_change] change,
    /// the `callback` (`fn`) is called with the result of the query.
    pub fn bind(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) + Send + Sync + 'static,
    ) {
        self.build().bind(callback)
    }

    /// Each time the components marked by [Self::track_change] change,
    /// the `callback` (`async fn`) is called with the result of the query.
    pub fn bind_async<R: Future<Output = ()>>(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) -> R + Copy + Send + Sync + 'static,
    ) {
        self.build().bind_async(callback)
    }

    fn build(self) -> QueryImpl<Components> {
        assert!(
            !self.1.is_empty(),
            "No components specified for tracking. Did you call `ChangeQuery::track_change`?"
        );
        QueryImpl::new(self.0.build_impl(&self.1, host::QueryEvent::Frame))
    }
}

/// An ECS query that calls a callback when its associated event occurs.
pub struct EventQuery<Components: ComponentsTuple + Copy + Clone + 'static>(
    QueryBuilderImpl<Components>,
    QueryEvent,
);
impl<Components: ComponentsTuple + Copy + Clone + 'static> EventQuery<Components> {
    /// Creates a new [EventQuery] that will find entities that have the specified `components`
    /// that will call its bound function when the `event` occurs.
    pub fn create(event: QueryEvent, components: Components) -> Self {
        Self(QueryBuilderImpl::new(components.as_indices()), event)
    }

    /// The entities must include the components in `requires`.
    pub fn requires(mut self, requires: impl ComponentsTuple) -> Self {
        self.0.requires(requires);
        self
    }

    /// The entities must not include the components in `excludes`.
    pub fn excludes(mut self, excludes: impl ComponentsTuple) -> Self {
        self.0.excludes(excludes);
        self
    }

    /// Each time the entity associated with `components` experiences the event,
    /// the `callback` (`fn`) is called with the result of the query.
    pub fn bind(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) + Send + Sync + 'static,
    ) {
        self.build().bind(callback)
    }

    /// Each time the entity associated with `components` experiences the event,
    /// the `callback` (`async fn`) is called with the result of the query.
    pub fn bind_async<R: Future<Output = ()>>(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) -> R + Copy + Send + Sync + 'static,
    ) {
        self.build().bind_async(callback)
    }

    fn build(self) -> QueryImpl<Components> {
        QueryImpl::new(self.0.build_impl(
            &[],
            match self.1 {
                QueryEvent::Spawn => host::QueryEvent::Spawn,
                QueryEvent::Despawn => host::QueryEvent::Despawn,
            },
        ))
    }
}

#[derive(Clone, Copy)]
struct QueryImpl<Components: ComponentsTuple + Copy + Clone + 'static>(
    u64,
    PhantomData<Components>,
);
impl<Components: ComponentsTuple + Copy + Clone + 'static> QueryImpl<Components> {
    fn new(id: u64) -> Self {
        Self(id, PhantomData)
    }

    fn evaluate(&self) -> Vec<(EntityId, Components::Data)> {
        host::query_eval(self.0)
            .into_iter()
            .map(|(id, components)| {
                (
                    id.from_bindgen(),
                    Components::from_component_types(components)
                        .expect("invalid type conversion on component query"),
                )
            })
            .collect()
    }

    fn bind(self, callback: impl Fn(Vec<(EntityId, Components::Data)>) + Send + Sync + 'static) {
        on(event::FRAME, move |_| {
            let results = self.evaluate();
            if !results.is_empty() {
                callback(results);
            }
            EventOk
        })
    }
    fn bind_async<R: Future<Output = ()>>(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) -> R + Copy + Send + Sync + 'static,
    ) {
        on_async(event::FRAME, move |_| async move {
            let results = self.evaluate();
            if !results.is_empty() {
                callback(results).await;
            }
            EventOk
        })
    }
}

struct QueryBuilderImpl<Components: ComponentsTuple + Copy + Clone + 'static> {
    components: Vec<u32>,
    include: Vec<u32>,
    exclude: Vec<u32>,
    _data: PhantomData<Components>,
}
impl<Components: ComponentsTuple + Copy + Clone + 'static> QueryBuilderImpl<Components> {
    fn new(components: Vec<u32>) -> QueryBuilderImpl<Components> {
        Self {
            components,
            include: vec![],
            exclude: vec![],
            _data: PhantomData,
        }
    }
    pub fn requires(&mut self, include: impl ComponentsTuple) {
        self.include.extend_from_slice(&include.as_indices());
    }
    pub fn excludes(&mut self, exclude: impl ComponentsTuple) {
        self.exclude.extend_from_slice(&exclude.as_indices());
    }
    fn build_impl(self, changed: &[u32], event: host::QueryEvent) -> u64 {
        host::entity_query2(
            host::Query {
                components: &self.components,
                include: &self.include,
                exclude: &self.exclude,
                changed,
            },
            event,
        )
    }
}
