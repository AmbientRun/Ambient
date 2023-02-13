
@group(0)
@binding(0)
var heightmap: texture_storage_2d_array<r32float, read>;
@group(0)
@binding(1)
var normalmap: texture_storage_2d<rgba32float, write>;


fn get_terrain_height(coord: vec2<i32>) -> f32 {
    return textureLoad(heightmap, coord, #ROCK_LAYER).r +
    textureLoad(heightmap, coord, #SOIL_LAYER).r;
}

fn get_normal(texcoord: vec2<i32>) -> vec3<f32> {
    let height = get_terrain_height(texcoord);

    let height_up = get_terrain_height(texcoord + vec2<i32>(0, -1));
    let height_down = get_terrain_height(texcoord + vec2<i32>(0, 1));
    let height_left = get_terrain_height(texcoord + vec2<i32>(-1, 0));
    let height_right = get_terrain_height(texcoord + vec2<i32>(1, 0));
    let scale = 1.;
    let normal_up = vec3<f32>(0., -1., (height_up - height) * scale);
    let normal_down = vec3<f32>(0.0, 1.0, (height_down - height)*scale);
    let normal_left = vec3<f32>(-1.0, 0.0, (height_left - height)*scale);
    let normal_right = vec3<f32>(1.0, 0.0, (height_right - height)*scale);
    let a = cross(normal_up, normal_right);
    let b = cross(normal_down, normal_left);
    let normal = normalize(a + b);
    return normal;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let normal = get_normal(vec2<i32>(global_id.xy));

    textureStore(normalmap, vec2<i32>(global_id.xy), vec4<f32>(normal, 0.));
}
