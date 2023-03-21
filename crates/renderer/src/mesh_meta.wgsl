struct MeshMetadata {
    base_offset: u32,
    skinned_offset: u32,
    index_offset: u32,

    index_count: u32,
};

@group(GLOBALS_BIND_GROUP)
@binding(MESH_METADATA_BINDING)
var<storage> mesh_metadatas: array<MeshMetadata>;
