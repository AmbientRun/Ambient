use crate::internal::wit;
use ambient_shared_types::procedural_storage_handle_definitions;
use paste::paste;
use serde::{Deserialize, Serialize};

macro_rules! make_procedural_storage_handles {
    ($($name:ident),*) => { paste!{$(
        #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
        /// Procedural storage handle type
        pub struct [<Procedural $name:camel Handle>](pub(crate) wit::types::Ulid);

        impl Default for [<Procedural $name:camel Handle>] {
            fn default() -> Self {
                Self((0, 0))
            }
        }
    )*}};
}

procedural_storage_handle_definitions!(make_procedural_storage_handles);
