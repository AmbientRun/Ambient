pub use crate::internal::component::{
    query::{
        change_query, despawn_query, query, spawn_query, ChangeQuery, EventQuery, GeneralQuery,
        GeneralQueryBuilder, QueryEvent, UntrackedChangeQuery,
    },
    Component, ComponentsTuple, Entity, EnumComponent, SupportedValue, UntypedComponent,
    __internal_get_component,
};

#[doc(hidden)]
pub use crate::internal::wit::component::Value as WitComponentValue;
