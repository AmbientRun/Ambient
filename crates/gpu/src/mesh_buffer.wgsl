
struct MeshMetadata {
    base_offset: u32,
    // normal_offset: u32,
    // tangent_offset: u32,
    // texcoord0_offset: u32,
    joint_offset: u32,
    weight_offset: u32,
    index_offset: u32,

    index_count: u32,
};


struct MeshMetadatas {
    data: array<MeshMetadata>,
};
