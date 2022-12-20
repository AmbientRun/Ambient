
@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_METADATA_BINDING)
var<storage> mesh_metadatas: MeshMetadatas;

@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_POSITION_BINDING)
var<storage> mesh_position: Vec3Buffer;
@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_NORMAL_BINDING)
var<storage> mesh_normal: Vec3Buffer;
@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_TANGENT_BINDING)
var<storage> mesh_tangent: Vec3Buffer;
@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_TEXCOORD0_BINDING)
var<storage> mesh_texcoord0: Vec2Buffer;
@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_JOINT_BINDING)
var<storage> mesh_joint: UVec4Buffer;
@group(#RESOURCES_BIND_GROUP)
@binding(#MESH_WEIGHT_BINDING)
var<storage> mesh_weight: Vec4Buffer;

fn get_raw_mesh_position(vertex_index: u32) -> vec3<f32> {
    return mesh_position.data[vertex_index];
}

fn get_raw_mesh_uv(vertex_index: u32) -> vec2<f32> {
    return mesh_texcoord0.data[vertex_index];
}

fn get_mesh_position(mesh_id: u32, vertex_index: u32) -> vec3<f32> {
    return mesh_position.data[mesh_metadatas.data[mesh_id].position_offset + vertex_index];
}
fn get_mesh_normal(mesh_id: u32, vertex_index: u32) -> vec3<f32> {
    return mesh_normal.data[mesh_metadatas.data[mesh_id].normal_offset + vertex_index];
}
fn get_mesh_tangent(mesh_id: u32, vertex_index: u32) -> vec3<f32> {
    return mesh_tangent.data[mesh_metadatas.data[mesh_id].tangent_offset + vertex_index];
}
fn get_mesh_texcoord0(mesh_id: u32, vertex_index: u32) -> vec2<f32> {
    return mesh_texcoord0.data[mesh_metadatas.data[mesh_id].texcoord0_offset + vertex_index];
}
fn get_mesh_joint(mesh_id: u32, vertex_index: u32) -> vec4<u32> {
    return mesh_joint.data[mesh_metadatas.data[mesh_id].joint_offset + vertex_index];
}
fn get_mesh_weight(mesh_id: u32, vertex_index: u32) -> vec4<f32> {
    return mesh_weight.data[mesh_metadatas.data[mesh_id].weight_offset + vertex_index];
}

@group(#RESOURCES_BIND_GROUP)
@binding(#SKINS_BINDING)
var<storage> skins: Mat4x4Buffer;
