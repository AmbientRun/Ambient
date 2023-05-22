use crate::internal::wit;

macro_rules! procedural_storage_handle {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
        /// Procedural storage handle type
        pub struct $name(wit::types::Ulid);

        impl Default for $name {
            fn default() -> Self {
                Self((0, 0))
            }
        }
    };
}

procedural_storage_handle!(ProceduralMeshHandle);
procedural_storage_handle!(ProceduralTextureHandle);
procedural_storage_handle!(ProceduralSamplerHandle);
procedural_storage_handle!(ProceduralMaterialHandle);
