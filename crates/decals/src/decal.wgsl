//
// Vertex shader
//

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) @interpolate(flat) entity_loc: vec2<u32>,
    @location(3) @interpolate(flat) instance_index: u32,
    @location(4) inv_local_to_world_0: vec4<f32>,
    @location(5) inv_local_to_world_1: vec4<f32>,
    @location(6) inv_local_to_world_2: vec4<f32>,
    @location(7) inv_local_to_world_3: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let primitive = primitives.data[in.instance_index];
    let entity_loc = primitive.xy;
    let mesh_index = get_entity_primitive_mesh(entity_loc, primitive.z);
    let mesh = get_mesh_base(mesh_index, in.vertex_index);
    let local_to_world = get_entity_mesh_to_world(entity_loc);
    let inv_local_to_world = inverse(local_to_world); // Todo: avoid inverse by moving world_to_local to global_params.
    let local_position = mesh.position;
    let world_position = local_to_world * vec4(local_position, 1.0);
    let position = global_params.projection_view * world_position;

    var out: VertexOutput;
    out.position = position;
    out.local_position = local_position;
    out.world_position = world_position;
    out.entity_loc = entity_loc;
    out.instance_index = in.instance_index;
    out.inv_local_to_world_0 = inv_local_to_world[0];
    out.inv_local_to_world_1 = inv_local_to_world[1];
    out.inv_local_to_world_2 = inv_local_to_world[2];
    out.inv_local_to_world_3 = inv_local_to_world[3];
    return out;
}

//
// Decals
//

struct Decal {
    material: MaterialInput,
}

fn decal(out: ptr<function, Decal>, in: VertexOutput) -> bool {
    let screen_size = vec2<f32>(textureDimensions(solids_screen_depth));
    let screen_tc = screen_pixel_to_uv(in.position.xy, screen_size);
    let screen_ndc = screen_uv_to_ndc(screen_tc);
    let screen_depth = get_solids_screen_depth(screen_ndc);
    if screen_depth == 0.0 {
        return false;
    }
    let world_position = project_point(global_params.inv_projection_view, vec3<f32>(screen_ndc.x, screen_ndc.y, screen_depth));
    let inv_local_to_world = mat4x4(
        in.inv_local_to_world_0,
        in.inv_local_to_world_1,
        in.inv_local_to_world_2,
        in.inv_local_to_world_3,
    );
    let local_position = project_point(inv_local_to_world, world_position);
    if local_position.x < -0.5 || local_position.x > 0.5 || local_position.y < -0.5 || local_position.y > 0.5 {
        return false;
    }
    let texcoord = vec2(local_position.xy + 0.5);
    let normal_matrix = mat3_from_quat(get_solids_screen_normal_quat(screen_ndc));

    var decal: Decal;
    decal.material.position = in.position;
    decal.material.texcoord = texcoord;
    decal.material.world_position = in.world_position.xyz;
    decal.material.normal = normal_matrix * vec3<f32>(0.0, 0.0, 1.0);
    decal.material.normal_matrix = normal_matrix;
    decal.material.instance_index = in.instance_index;
    decal.material.entity_loc = in.entity_loc;
    decal.material.local_position = in.local_position;
    *out = decal;

    return true;
}

//
// Fragment shader - shadows
//

@fragment
fn fs_shadow_main(in: VertexOutput) {
    // Decals don't cast shadows.
}

//
// Fragment shader - lit / unlit
//

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@fragment
fn fs_forward_lit_main(in: VertexOutput) -> FragmentOutput {
    var decal: Decal;
    if decal(&decal, in) {
        var out: FragmentOutput;
        out.color = shading(get_material(decal.material), in.world_position);
        return out;
    } else {
        discard;
    }
}

@fragment
fn fs_forward_unlit_main(in: VertexOutput) -> FragmentOutput {
    var decal: Decal;
    if decal(&decal, in) {
        var out: FragmentOutput;
        out.color = shading(get_material(decal.material), in.world_position);
        return out;
    } else {
        discard;
    }
}

//
// Fragment shader - outlines
//

fn get_outline(instance_index: u32) -> vec4<f32> {
    let entity_loc = primitives.data[instance_index].xy;
    return get_entity_outline_or(entity_loc, vec4<f32>(0., 0., 0., 0.));
}

@fragment
fn fs_outlines_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var decal: Decal;
    if decal(&decal, in) {
        return get_outline(in.instance_index);
    } else {
        discard;
    }
}
