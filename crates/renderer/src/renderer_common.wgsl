

@group(PRIMITIVES_BIND_GROUP)
@binding(0)
var<storage> primitives: UVec4Buffer;

struct ModelToWorld {
    local: vec4<f32>,
    pos: vec4<f32>,
    normal: vec3<f32>,
    tangent: vec3<f32>,
    texcoord: vec2<f32>,
}


/// Transform a vertex from model space to world space by applying
// joint matrices (if applicable) and transformation matrices
fn model_to_world(loc: vec2<u32>, mesh_index: u32, vertex_index: u32) -> ModelToWorld {
    let model = get_entity_mesh_to_world(loc);

    let mesh = get_mesh_base(mesh_index, vertex_index);

    let pos = vec4<f32>(mesh.position.xyz, 1.0);
    let normal = vec4<f32>(mesh.normal.xyz, 0.0);
    let tangent = vec4<f32>(mesh.tangent.xyz, 0.0);
    let texcoord: vec2<f32> = mesh.texcoord0;

    var result: ModelToWorld;

    if has_entity_skin(loc) {

        let skin = get_mesh_skinned(mesh_index, vertex_index);
        let joint = skin.joint;
        let weights = skin.weights;

        let skin_offset = u32(get_entity_skin(loc).x);

        let ltw_x: mat4x4<f32> = skins.data[skin_offset + joint.x];
        let ltw_y: mat4x4<f32> = skins.data[skin_offset + joint.y];
        let ltw_z: mat4x4<f32> = skins.data[skin_offset + joint.z];
        let ltw_w: mat4x4<f32> = skins.data[skin_offset + joint.w];

        var total_pos = vec4<f32>(0.0);
        var total_norm = vec4<f32>(0.0);
        var total_tangent = vec4<f32>(0.0);

        // Normalize the weights
        let mesh_weight = weights / dot(weights, vec4<f32>(1.0));

        total_pos = total_pos + (ltw_x * pos) * mesh_weight.x;
        total_pos = total_pos + (ltw_y * pos) * mesh_weight.y;
        total_pos = total_pos + (ltw_z * pos) * mesh_weight.z;
        total_pos = total_pos + (ltw_w * pos) * mesh_weight.w;

        total_pos.w = 1.0;

        total_norm = total_norm + (ltw_x * normal) * mesh_weight.x;
        total_norm = total_norm + (ltw_y * normal) * mesh_weight.y;
        total_norm = total_norm + (ltw_z * normal) * mesh_weight.z;
        total_norm = total_norm + (ltw_w * normal) * mesh_weight.w;


        total_tangent = total_tangent + (ltw_x * tangent) * mesh_weight.x;
        total_tangent = total_tangent + (ltw_y * tangent) * mesh_weight.y;
        total_tangent = total_tangent + (ltw_z * tangent) * mesh_weight.z;
        total_tangent = total_tangent + (ltw_w * tangent) * mesh_weight.w;

        result.local = total_pos;
        result.pos = model * total_pos;
        result.normal = normalize((model * total_norm).xyz);
        result.tangent = normalize((model * total_tangent).xyz);
        result.texcoord = texcoord;
    } else {
        result.local = pos;
        result.pos = model * pos;
        result.normal = normalize((model * normal).xyz);
        result.tangent = normalize((model * tangent).xyz);
        result.texcoord = texcoord;
    }

    return result;
}

fn get_entity_primitive_mesh(loc: vec2<u32>, index: u32) -> u32 {
    var meshes = get_entity_gpu_primitives_mesh(loc);
    // NOTE: Tint does not appear to support non-constant expression indexing on stack based matrices.
    // The specification is not entirely clear on this either.
    // <https://github.com/gfx-rs/naga/issues/920>
    // <https://www.w3.org/TR/WGSL/#matrix-access-expr>
    // TODO: find a way to inspect and debug Tint generated code
    if index == 0u {
        return bitcast<u32>(meshes[0][0]);
    } else if index == 1u {
        return bitcast<u32>(meshes[0][1]);
    } else if index == 2u {
        return bitcast<u32>(meshes[0][2]);
    } else if index == 3u {
        return bitcast<u32>(meshes[0][3]);
    } else if index == 4u {
        return bitcast<u32>(meshes[1][0]);
    } else if index == 5u {
        return bitcast<u32>(meshes[1][1]);
    } else if index == 6u {
        return bitcast<u32>(meshes[1][2]);
    } else if index == 7u {
        return bitcast<u32>(meshes[1][3]);
    } else if index == 8u {
        return bitcast<u32>(meshes[2][0]);
    } else if index == 9u {
        return bitcast<u32>(meshes[2][1]);
    } else if index == 10u {
        return bitcast<u32>(meshes[2][2]);
    } else if index == 11u {
        return bitcast<u32>(meshes[2][3]);
    } else if index == 12u {
        return bitcast<u32>(meshes[3][0]);
    } else if index == 13u {
        return bitcast<u32>(meshes[3][1]);
    } else if index == 14u {
        return bitcast<u32>(meshes[3][2]);
    } else if index == 15u {
        return bitcast<u32>(meshes[3][3]);
    } else {
        // unreachable
        return 1000u;
    }
}

