
struct Params {
    heightmap_world_position: vec2<f32>,
    heightmap_world_size: vec2<f32>,
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

fn generate_level(cell: vec2<i32>, p: vec2<f32>, layer: i32, base: f32, amount: f32) {
    var height: f32 = textureLoad(heightmap, cell, layer).x;

    let power = 1.;
    let persistence = 0.4;
    let lacunarity = 2.;

    var max_height: f32 = 0.;
    var amplitude: f32 = 1.;
    var scale: f32 = 5. / 256.;
    for(var i: i32 = 0; i < i32(arrayLength(&offsets.data)); i = i + 1) {
        let r = p * scale + offsets.data[i];
        height = height + amplitude * snoise2d(r);
        max_height = max_height + amplitude;
        scale = scale * lacunarity;
        amplitude = amplitude * persistence;
    }
    height = interpolate_1_1(height, -max_height, max_height, 0., 1.);
    height = pow(height, power) * amount + base;

    textureStore(heightmap, cell, layer, vec4<f32>(height, 0., 0., 0.));
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = vec2<f32>(textureDimensions(heightmap));

    let p = vec2<f32>(global_id.xy) * params.heightmap_world_size / (size - 1.) + params.heightmap_world_position;
    generate_level(vec2<i32>(global_id.xy), p, #ROCK_LAYER, 20., 1.);
    generate_level(vec2<i32>(global_id.xy), p + vec2<f32>(2000., 2000.), #SOIL_LAYER, 20., 1.);
}
