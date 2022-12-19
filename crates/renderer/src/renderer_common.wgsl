

@group(#PRIMITIVES_BIND_GROUP)
@binding(0)
var<storage> primitives: UVec4Buffer;

struct ModelToWorld {
    local: vec4<f32>,
    pos: vec4<f32>,
    normal: vec3<f32>,
    tangent: vec3<f32>,
}

/// Transform a vertex from model space to world space by applying
// joint matrices (if applicable) and transformation matrices
fn model_to_world(loc: vec2<u32>, mesh_index: u32, vertex_index: u32) -> ModelToWorld {
    let model = get_entity_mesh_to_world(loc);
    let pos = vec4<f32>(get_mesh_position(mesh_index, vertex_index), 1.0);
    let normal = vec4<f32>(get_mesh_normal(mesh_index, vertex_index), 0.0);
    let tangent = vec4<f32>(get_mesh_tangent(mesh_index, vertex_index), 0.0);

    if (has_entity_skin(loc)) {
        let joint = get_mesh_joint(mesh_index, vertex_index);
        let weight = get_mesh_weight(mesh_index, vertex_index);
        let skin_offset = get_entity_skin(loc);

        let ltw_x: mat4x4<f32> = skins.data[skin_offset + joint.x]; 
        let ltw_y: mat4x4<f32> = skins.data[skin_offset + joint.y]; 
        let ltw_z: mat4x4<f32> = skins.data[skin_offset + joint.z]; 
        let ltw_w: mat4x4<f32> = skins.data[skin_offset + joint.w]; 

        var total_pos     = vec4<f32>(0.0);
        var total_norm    = vec4<f32>(0.0);
        var total_tangent = vec4<f32>(0.0);

        // Normalize the weights
        let weight = weight / dot(weight, vec4<f32>(1.0));

        total_pos = total_pos + (ltw_x * pos) * weight.x;
        total_pos = total_pos + (ltw_y * pos) * weight.y;
        total_pos = total_pos + (ltw_z * pos) * weight.z;
        total_pos = total_pos + (ltw_w * pos) * weight.w;

        total_pos.w = 1.0;

        total_norm = total_norm + (ltw_x * normal) * weight.x;
        total_norm = total_norm + (ltw_y * normal) * weight.y;
        total_norm = total_norm + (ltw_z * normal) * weight.z;
        total_norm = total_norm + (ltw_w * normal) * weight.w;


        total_tangent = total_tangent + (ltw_x * tangent) * weight.x;
        total_tangent = total_tangent + (ltw_y * tangent) * weight.y;
        total_tangent = total_tangent + (ltw_z * tangent) * weight.z;
        total_tangent = total_tangent + (ltw_w * tangent) * weight.w;

        return ModelToWorld(total_pos, model * total_pos, normalize((model * total_norm).xyz), normalize((model * normalize(tangent)).xyz));
    } else {
        return ModelToWorld(pos, model * pos, normalize((model * normal).xyz), normalize((model * tangent).xyz));
    }
}
