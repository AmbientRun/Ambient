struct Params {
    camera: u32,
};

@group(2)
@binding(0)
var<uniform> params: Params;

struct CollectPrimitive {
    entity_loc: vec2<u32>,
    primitive_index: u32,
    material_index: u32,
};
struct CollectPrimitives { data: array<CollectPrimitive>, };
@group(2)
@binding(1)
var<storage> input_primitives: CollectPrimitives;

struct DrawIndexedIndirect {
    vertex_count: u32,
    instance_count: u32,
    base_index: u32,
    vertex_offset: i32,
    base_instance: u32,
};

struct Commands {
    data: array<DrawIndexedIndirect>,
};
@group(2)
@binding(2)
var<storage, read_write> output_commands: Commands;

struct AtomicU32Buffer { data: array<atomic<u32>>, };
@group(2)
@binding(3)
var<storage, read_write> output_counts: AtomicU32Buffer;

@group(2)
@binding(4)
var<storage> material_layouts: UVec2Buffer;

fn get_entity_primitive(loc: vec2<u32>, index: u32) -> vec2<u32> {
    let i = index >> 2u;
    let j = index & 3u;

    var meshes = get_entity_gpu_primitives_mesh(loc);
    var lods = get_entity_gpu_primitives_lod(loc);

    let mesh = bitcast<u32>(meshes[i][j]);
    let lod = bitcast<u32>(lods[i][j]);

    return vec2<u32>(mesh, lod);
}


fn is_visible(entity_loc: vec2<u32>, primitive_lod: u32) -> bool {
    var visibility_from: vec2<u32> = entity_loc;

    if has_entity_visibility_from(entity_loc) {
        let visibility_from_raw = get_entity_visibility_from(entity_loc);
        // reinterpret floats as u32
        visibility_from = bitcast<vec4<u32>>(visibility_from_raw).xy;
    }

    // let entity_lod = u32(get_entity_gpu_lod_or(visibility_from, 0.0).x);
    let flod = get_entity_gpu_lod_or(visibility_from, vec4<f32>(0.0));
    let entity_lod = u32(flod.x);

    if entity_lod != primitive_lod {
        return false;
    }

    if has_entity_renderer_cameras_visible(visibility_from) {
        var cameras = get_entity_renderer_cameras_visible(visibility_from);
        let camera_i = params.camera >> 2u;
        let camera_j = params.camera & 3u;

        return cameras[camera_i][camera_j] > 0.0;
    } else {
        return true;
    }
}

@compute
@workgroup_size(COLLECT_WORKGROUP_SIZE)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let chunk = COLLECT_CHUNK_SIZEu * COLLECT_WORKGROUP_SIZEu;
    let index = global_id.y * chunk + global_id.x;

    if index >= arrayLength(&input_primitives.data) {
        // return;
    }


    let primitive = input_primitives.data[index];
    let material_layout = material_layouts.data[primitive.material_index];
    if index < material_layout.x || index >= material_layout.x + material_layout.y {
        // return;
    }


    var entity_primitive = get_entity_primitive(primitive.entity_loc, primitive.primitive_index);
    let mesh_index = entity_primitive.x;
    let primitive_lod = entity_primitive.y;

    let out_offset = atomicAdd(&output_counts.data[primitive.material_index], 1u);

    let out_index = material_layout.x + out_offset;
    let mesh = mesh_metadatas[mesh_index];

    output_commands.data[out_index].vertex_count = mesh.index_count;
    output_commands.data[out_index].instance_count = 1u;
    output_commands.data[out_index].base_index = mesh.index_offset;
    output_commands.data[out_index].base_instance = index;
}
