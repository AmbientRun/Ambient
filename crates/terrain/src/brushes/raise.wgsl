
struct Params {
    brush: Brush,
    heightmap_world_position: vec2<f32>,
    heightmap_world_texel_size: f32,
    layer: i32,
};
@group(0)
@binding(0)
var heightmap: texture_storage_2d_array<r32float, read_write>;

@group(0)
@binding(1)
var<uniform> params: Params;

struct Vec2Buffer { data: array<vec2<f32>>, };
@group(0)
@binding(2)
var<storage> offsets: Vec2Buffer;


@group(0)
@binding(3)
var noise_texture: texture_2d<f32>;

@group(0)
@binding(4)
var noise_sampler: sampler;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let p = vec2<f32>(global_id.xy) * params.heightmap_world_texel_size + params.heightmap_world_position;
    var height: f32 = textureLoad(heightmap, vec2<i32>(global_id.xy), params.layer).x;

    let brush_strength = get_brush_strength(params.brush, p);

    let p = p * 0.01;

    height = height + brush_strength * mix(0.5, 1., textureSampleLevel(noise_texture, noise_sampler, offsets.data[0] + p * 0.1, 0.).r);
    height = max(height, 0.);

    textureStore(heightmap, vec2<i32>(global_id.xy), params.layer, vec4<f32>(height, 0., 0., 0.));

    let hardness = pow(textureSampleLevel(noise_texture, noise_sampler, offsets.data[1] + p * 0.02, 0.).r, 1.);
    textureStore(heightmap, vec2<i32>(global_id.xy), #HARDNESS_LAYER, vec4<f32>(hardness, 0., 0., 0.));

    let strata_amplitude = smoothstep(0.4, 0.7, textureSampleLevel(noise_texture, noise_sampler, offsets.data[2] + p * 0.01, 0.).r);
    textureStore(heightmap, vec2<i32>(global_id.xy), #HARDNESS_STRATA_AMOUNT_LAYER, vec4<f32>(strata_amplitude, 0., 0., 0.));

    let strata_wavelength = mix(60., 200., textureSampleLevel(noise_texture, noise_sampler, offsets.data[3] + p * 0.05, 0.).r);
    textureStore(heightmap, vec2<i32>(global_id.xy), #HARDNESS_STRATA_WAVELENGTH_LAYER, vec4<f32>(strata_wavelength, 0., 0., 0.));

}
