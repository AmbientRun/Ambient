use ulid::Ulid;

macro_rules! procedural_storage_handle {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
        /// Procedural storage handle type
        pub struct $name(Ulid);

        impl $name {
            /// Creates a handle
            pub fn new() -> Self {
                Self(Ulid::new())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(Ulid::nil())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Ulid> for $name {
            fn from(ulid: Ulid) -> Self {
                Self(ulid)
            }
        }

        impl From<$name> for Ulid {
            fn from(handle: $name) -> Self {
                handle.0
            }
        }
    };
}

procedural_storage_handle!(ProceduralMeshHandle);
procedural_storage_handle!(ProceduralTextureHandle);
procedural_storage_handle!(ProceduralSamplerHandle);
procedural_storage_handle!(ProceduralMaterialHandle);
