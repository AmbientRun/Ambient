
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) @interpolate(flat) instance_index: u32,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
    @location(5) world_normal: vec3<f32>,
    @location(6) local_position: vec3<f32>,
};

@vertex
fn vs_main(@builtin(instance_index) instance_index: u32, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let primitive = primitives.data[instance_index];
    let entity_loc = primitive.xy;
    let mesh_index = get_entity_primitive_mesh(entity_loc, primitive.z);

    let world = model_to_world(entity_loc, mesh_index, vertex_index);
    out.instance_index = instance_index;
    out.texcoord = world.texcoord;

    out.world_normal = world.normal;
    out.world_tangent = world.tangent;
    out.world_bitangent = cross(world.normal, world.tangent);
    out.world_position = world.pos;
    out.local_position = world.local.xyz;

    let clip = global_params.projection_view * world.pos;

    out.position = clip;
    return out;
}

fn get_material_in(in: VertexOutput, is_front: bool) -> MaterialInput {
    var material_in: MaterialInput;
    material_in.position = in.position;
    material_in.texcoord = in.texcoord;
    material_in.world_position = in.world_position.xyz / in.world_position.w;
    material_in.normal = in.world_normal;
    material_in.normal_matrix = mat3x3<f32>(
        in.world_tangent,
        in.world_bitangent,
        in.world_normal
    );
    material_in.instance_index = in.instance_index;
    material_in.entity_loc = primitives.data[in.instance_index].xy;
    material_in.local_position = in.local_position;
    return material_in;
}

@fragment
fn fs_shadow_main(in: VertexOutput, @builtin(front_facing) is_front: bool) {
    var material = get_material(get_material_in(in, is_front));

    if material.opacity < material.alpha_cutoff {
        discard;
    }
}

fn get_outline(instance_index: u32) -> vec4<f32> {
    let entity_loc = primitives.data[instance_index].xy;
    return get_entity_outline_or(entity_loc, vec4<f32>(0., 0., 0., 0.));
}

@fragment
fn fs_forward_lit_main(in: VertexOutput, @builtin(front_facing) is_front: bool) -> MainFsOut {
    let material_in = get_material_in(in, is_front);
    var material = get_material(material_in);

    if material.opacity < material.alpha_cutoff {
        discard;
    }

    if !is_front {
        material.normal = -material.normal;
    }

    material.normal = normalize(material.normal);

    return MainFsOut(
        shading(material, in.world_position),
        quat_from_mat3(material_in.normal_matrix)
    );
}

@fragment
fn fs_forward_unlit_main(in: VertexOutput, @builtin(front_facing) is_front: bool) -> MainFsOut {
    let material_in = get_material_in(in, is_front);
    var material = get_material(material_in);

    if material.opacity < material.alpha_cutoff {
        discard;
    }

    return MainFsOut(
        vec4<f32>(material.base_color, material.opacity),
        quat_from_mat3(material_in.normal_matrix)
    );
}

@fragment
fn fs_outlines_main(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    var material = get_material(get_material_in(in, is_front));

    if material.opacity < material.alpha_cutoff {
        discard;
    }
    return get_outline(in.instance_index);
}
