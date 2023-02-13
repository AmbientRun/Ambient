
struct Params {
    brush: Brush,
    start_texel: vec2<i32>,
    heightmap_world_position: vec2<f32>,
    heightmap_world_texel_size: f32,
};
@group(0)
@binding(0)
var heightmap: texture_storage_2d_array<r32float, read_write>;

@group(0)
@binding(1)
var<uniform> params: Params;

@group(0)
@binding(2)
var start_heightmap: texture_storage_2d_array<r32float, read>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let layer = i32(global_id.z);

    let p = vec2<f32>(global_id.xy) * params.heightmap_world_texel_size + params.heightmap_world_position;
    var sample_height: f32 = textureLoad(start_heightmap, params.start_texel, layer).x;
    var our_height: f32 = textureLoad(heightmap, vec2<i32>(global_id.xy), layer).x;
    var delta = sample_height - our_height;

    // Remap the brush strength to something a little more reasonable,
    // and ensure that we don't apply extreme changes.
    var brush_strength = get_brush_strength(params.brush, p) * 0.04;
    var target_height = our_height + clamp(delta * brush_strength, -10.0, 10.0);

    textureStore(heightmap, vec2<i32>(global_id.xy), layer, vec4<f32>(target_height, 0., 0., 0.));
}
