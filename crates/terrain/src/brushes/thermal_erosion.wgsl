
struct Params {
    heightmap_world_position: vec2<f32>,
    heightmap_world_size: vec2<f32>,
    heightmap_texture_size: vec2<i32>,
    brush_position: vec2<f32>,
    brush_radius: f32,
    frame: i32,
};
@group(0)
@binding(0)
var heightmap: texture_storage_2d_array<r32float, read_write>;
@group(0)
@binding(1)
var<uniform> params: Params;

struct Sample {
    rock: f32,
    soil: f32,
    depth: f32,
};
fn get_sample(cell: vec2<i32>) -> Sample {
    var res: Sample;
    res.rock = textureLoad(heightmap, cell, ROCK_LAYER).r;
    res.soil = textureLoad(heightmap, cell, SOIL_LAYER).r;
    res.depth = res.rock + res.soil;
    return res;
}

fn thermal_erode(local_depth: f32, neighbor_depth: f32, local: f32, neighbor: f32, viscosity: f32, brush_strength: f32) -> f32 {
    let d = local_depth - neighbor_depth;
    if atan(abs(d)) > viscosity {
        if d > 0. {
            return local - min(brush_strength * d / 4., local);
        } else {
            return local + min(brush_strength * -d / 4., neighbor);
        }
    } else {
        return local;
    }
}

fn deg_to_rad(deg: f32) -> f32 {
    return deg * 3.14159 / 180.;
}

TERRAIN_FUNCS
GET_HARDNESS

@compute
@workgroup_size(32)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell = vec2<i32>(global_id.xy);
    let size = vec2<f32>(params.heightmap_texture_size);
    let p = vec2<f32>(global_id.xy) * params.heightmap_world_size / (size - 1.) + params.heightmap_world_position;

    let t = vec2<f32>(global_id.xy);
    let border_dist = min(
        min(t.x, t.y),
        min(size.x - 1. - t.x, size.y - 1. - t.y)
    );
    let brush_strength = smoothstep(0., 50., max(0., border_dist - 1.));
    // let d = length(p - params.brush_position);
    // let brush_strength = smoothstep(1., 0.8, min(1., d / params.brush_radius));

    var dir: vec2<i32>;
    if params.frame % 4 == 0 {
        dir = vec2<i32>(-1, 0);
    } else if params.frame % 4 == 1 {
        dir = vec2<i32>(1, 0);
    } else if params.frame % 4 == 2 {
        dir = vec2<i32>(0, -1);
    } else if params.frame % 4 == 3 {
        dir = vec2<i32>(0, 1);
    }

    var local = get_sample(cell);
    var neighbor = get_sample(cell + dir);

    if local.soil > 0.1 {
        let soil = thermal_erode(local.depth, neighbor.depth, local.soil, neighbor.soil, deg_to_rad(10.), brush_strength);
        textureStore(heightmap, cell, SOIL_LAYER, vec4<f32>(soil, 0., 0., 0.));
    } else {
        let hardness = get_hardness(cell, local.depth + f32(TERRAIN_BASE));
        let viscosity = mix(deg_to_rad(89.), deg_to_rad(45.), hardness);
        let rock = thermal_erode(local.depth, neighbor.depth, local.rock, neighbor.rock, viscosity, brush_strength);
        textureStore(heightmap, cell, ROCK_LAYER, vec4<f32>(rock, 0., 0., 0.));
    }
}
