
struct GpuUpdateChunksBuffer {
    data: array<vec4<u32>>,
};

@group(GPU_ECS_UPDATE_BIND_GROUP)
@binding(0)
var<storage, read> gpu_update_chunks: GpuUpdateChunksBuffer;

@compute
@workgroup_size(GPU_ECS_WORKGROUP_SIZE)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let chunk = gpu_update_chunks.data[global_id.y];
    let entity_loc = vec2<u32>(chunk.x, chunk.y + global_id.x);
    if entity_loc.y < chunk.z {
        UPDATE_BODY
    }
}
