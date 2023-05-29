use paste::paste;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[macro_export]
macro_rules! procedural_storage_handle_definitions {
    ($macro_to_instantiate:ident) => {
        // Handle names must be in snake_case.
        $macro_to_instantiate!(mesh, texture, sampler, material);
    };
}

macro_rules! make_procedural_storage_handles {
    ($($name:ident),*) => { paste!{$(
        #[derive(
            Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize,
        )]
        pub struct [<Procedural $name:camel Handle>](Ulid);

        impl Default for [<Procedural $name:camel Handle>] {
            fn default() -> Self {
                Self(Ulid::nil())
            }
        }

        impl std::fmt::Display for [<Procedural $name:camel Handle>] {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, concat!(stringify!([<Procedural $name:camel Handle>]), "({})"), self.0)
            }
        }

        impl From<Ulid> for [<Procedural $name:camel Handle>] {
            fn from(ulid: Ulid) -> Self {
                Self(ulid)
            }
        }

        impl From<[<Procedural $name:camel Handle>]> for Ulid {
            fn from(handle: [<Procedural $name:camel Handle>]) -> Self {
                handle.0
            }
        }
    )*}};
}

procedural_storage_handle_definitions!(make_procedural_storage_handles);
