struct MeshBase {
    position: vec4<f32>,
    normal: vec4<f32>,
    tangent: vec4<f32>,
    texcoord0: vec2<f32>,
}

@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_BASE_BINDING)
var<storage> mesh_base: array<MeshBase>;

@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_JOINT_BINDING)
var<storage> mesh_joint: UVec4Buffer;

@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_WEIGHT_BINDING)
var<storage> mesh_weight: Vec4Buffer;

@group(#RESOURCES_BIND_GROUP)
@binding(#SKINS_BINDING)
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

fn get_mesh_position(mesh_id: u32, vertex_index: u32) -> vec3<f32> {
    return get_mesh_base(mesh_id, vertex_index).position.xyz;
}

fn get_mesh_normal(mesh_id: u32, vertex_index: u32) -> vec3<f32> {
    return get_mesh_base(mesh_id, vertex_index).normal.xyz;
}

fn get_mesh_tangent(mesh_id: u32, vertex_index: u32) -> vec3<f32> {
    return get_mesh_base(mesh_id, vertex_index).tangent.xyz;

}

fn get_mesh_texcoord0(mesh_id: u32, vertex_index: u32) -> vec2<f32> {
    return get_mesh_base(mesh_id, vertex_index).texcoord0;
}

fn get_mesh_joint(mesh_id: u32, vertex_index: u32) -> vec4<u32> {
    return mesh_joint.data[mesh_metadatas[mesh_id].joint_offset + vertex_index];
}

fn get_mesh_weight(mesh_id: u32, vertex_index: u32) -> vec4<f32> {
    return mesh_weight.data[mesh_metadatas[mesh_id].weight_offset + vertex_index];
}
