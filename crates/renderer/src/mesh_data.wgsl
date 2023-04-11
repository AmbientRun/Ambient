struct MeshBase {
    position: vec3<f32>,
    normal: vec3<f32>,
    tangent: vec3<f32>,
    texcoord0: vec2<f32>,
}

struct MeshSkinned {
    joint: vec4<u32>,
    weights: vec4<f32>,
}

@group(GLOBALS_BIND_GROUP)
@binding(MESH_BASE_BINDING)
var<storage> mesh_base: array<MeshBase>;

@group(GLOBALS_BIND_GROUP)
@binding(MESH_SKIN_BINDING)
var<storage> mesh_skinned: array<MeshSkinned>;

@group(GLOBALS_BIND_GROUP)
@binding(SKINS_BINDING)
var<storage> skins: Mat4x4Buffer;


fn get_raw_mesh_position(vertex_index: u32) -> vec3<f32> {
    return mesh_base[vertex_index].position.xyz;
}

fn get_raw_mesh_uv(vertex_index: u32) -> vec2<f32> {
    return mesh_base[vertex_index].texcoord0;
}

fn get_mesh_base(mesh_id: u32, vertex_index: u32) -> MeshBase {
    return mesh_base[mesh_metadatas[mesh_id].base_offset + vertex_index];
}

fn get_mesh_skinned(mesh_id: u32, vertex_index: u32) -> MeshSkinned {
    return mesh_skinned[mesh_metadatas[mesh_id].skinned_offset + vertex_index];
}
