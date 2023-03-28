use std::marker::PhantomData;

use crate::{
    event,
    global::{on, CallbackReturn, EntityId, OkEmpty},
    internal::{component::ComponentsTuple, conversion::FromBindgen, wit},
    prelude::OnHandle,
};

/// Creates a new [GeneralQueryBuilder] that will find entities that have the specified `components`
/// and can be [built](GeneralQueryBuilder::build) to create a [GeneralQuery].
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
) -> UntrackedChangeQuery<Components> {
    UntrackedChangeQuery::create(components)
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
    /// When this collection of components is spawned.
    Spawn,
    /// When this collection of components is despawned.
    Despawn,
}

#[derive(Clone, Copy)]
/// An ECS query used to find entities in the world.
pub struct GeneralQuery<Components: ComponentsTuple + Copy + Clone + 'static>(
    QueryImpl<Components>,
);
impl<Components: ComponentsTuple + Copy + Clone + 'static> GeneralQuery<Components> {
    /// Creates a new [GeneralQueryBuilder] that will find entities that have the specified `components`
    /// and can be [built](GeneralQueryBuilder::build) to create a [GeneralQuery].
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
    pub fn each_frame<R: CallbackReturn>(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) -> R + 'static,
    ) -> OnHandle {
        self.0.bind(callback)
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
            self.0.build_impl(&[], wit::component::QueryEvent::Frame),
        ))
    }
}

/// An ECS query that will call a callback when entities containing components
/// marked with [Self::track_change] have those components change. This type
/// represents a query that has not had any components marked for tracking yet.
///
/// Note that this cannot be built without calling [Self::track_change]
/// at least once.
pub struct UntrackedChangeQuery<Components: ComponentsTuple + Copy + Clone + 'static>(
    QueryBuilderImpl<Components>,
    Vec<u32>,
);
impl<Components: ComponentsTuple + Copy + Clone + 'static> UntrackedChangeQuery<Components> {
    /// Creates a new query that will find entities that have the specified `components`.
    /// It will call its bound function when components marked by [Self::track_change] change.
    pub fn create(components: Components) -> Self {
        Self(QueryBuilderImpl::new(components.as_indices()), vec![])
    }

    /// The query will return results when these components change values.
    ///
    /// This converts this type to a [ChangeQuery], which can be used to bind a callback.
    pub fn track_change(self, changes: impl ComponentsTuple) -> ChangeQuery<Components> {
        let cqwt = ChangeQuery(self);
        cqwt.track_change(changes)
    }
}

/// A partially-built ECS query that calls a callback when entities containing components
/// marked with [Self::track_change] have those components change.
///
/// This cannot be constructed without first going through [UntrackedChangeQuery] or
/// [change_query].
pub struct ChangeQuery<Components: ComponentsTuple + Copy + Clone + 'static>(
    UntrackedChangeQuery<Components>,
);
impl<Components: ComponentsTuple + Copy + Clone + 'static> ChangeQuery<Components> {
    /// The entities must include the components in `requires`.
    pub fn requires(mut self, requires: impl ComponentsTuple) -> Self {
        self.0 .0.requires(requires);
        self
    }

    /// The entities must not include the components in `exclude`.
    pub fn excludes(mut self, excludes: impl ComponentsTuple) -> Self {
        self.0 .0.excludes(excludes);
        self
    }

    /// The query will return results when these components change values.
    ///
    /// Note that this does *not* implicitly [requires](Self::requires) the components; this allows you to track
    /// changes for entities that do not have all of the tracked components.
    pub fn track_change(mut self, changes: impl ComponentsTuple) -> Self {
        self.0 .1.extend_from_slice(&changes.as_indices());
        self
    }

    /// Each time the components marked by [Self::track_change] change,
    /// the `callback` (`fn`) is called with the result of the query.
    pub fn bind<R: CallbackReturn>(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) -> R + 'static,
    ) -> OnHandle {
        self.build().bind(callback)
    }

    fn build(self) -> QueryImpl<Components> {
        QueryImpl::new(
            self.0
                 .0
                .build_impl(&self.0 .1, wit::component::QueryEvent::Frame),
        )
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
    pub fn bind<R: CallbackReturn>(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) -> R + 'static,
    ) -> OnHandle {
        self.build().bind(callback)
    }

    fn build(self) -> QueryImpl<Components> {
        QueryImpl::new(self.0.build_impl(
            &[],
            match self.1 {
                QueryEvent::Spawn => wit::component::QueryEvent::Spawn,
                QueryEvent::Despawn => wit::component::QueryEvent::Despawn,
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
        wit::component::query_eval(self.0)
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

    fn bind<R: CallbackReturn>(
        self,
        callback: impl Fn(Vec<(EntityId, Components::Data)>) -> R + 'static,
    ) -> OnHandle {
        on(event::FRAME, move |_| {
            let results = self.evaluate();
            if !results.is_empty() {
                callback(results).into_result()?;
            }
            OkEmpty
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
    fn build_impl(self, changed: &[u32], event: wit::component::QueryEvent) -> u64 {
        wit::component::query(
            wit::component::QueryBuild {
                components: &self.components,
                include: &self.include,
                exclude: &self.exclude,
                changed,
            },
            event,
        )
    }
}
