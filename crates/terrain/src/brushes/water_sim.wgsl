
struct Params {
    gravity: f32,
    frame: i32,
};
@group(0)
@binding(0)
var heightmap: texture_storage_2d_array<r32float, read_write>;
@group(0)
@binding(1)
var<uniform> params: Params;

fn get_height(coord: vec2<i32>) -> f32 {
    return textureLoad(heightmap, coord, #ROCK_LAYER).r +
        textureLoad(heightmap, coord, #SOIL_LAYER).r +
        textureLoad(heightmap, coord, #WATER_LAYER).r;
}

const d_t = 0.02;

fn calc_next_flux(cell: vec2<i32>, delta: vec2<i32>, layer: i32, cell_height: f32) -> f32 {
    let f = textureLoad(heightmap, cell, layer).r;
    let h = clamp(cell_height - get_height(cell + delta), -2., 2.);
    let next_f = max(0., f + d_t * params.gravity * h);
    return next_f;
}


@compute
@workgroup_size(32)
fn rain(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell = vec2<i32>(global_id.xy);
    var water: f32 = textureLoad(heightmap, cell, #WATER_LAYER).r;
    let rain = 0.012;
    water = water + d_t * rain;
    textureStore(heightmap, cell, #WATER_LAYER, vec4<f32>(water, 0., 0., 0.));
}

// Based on https://old.cescg.org/CESCG-2011/papers/TUBudapest-Jako-Balazs.pdf

@compute
@workgroup_size(32)
fn flux(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell = vec2<i32>(global_id.xy);
    let cell_height = get_height(cell);

    let water = textureLoad(heightmap, cell, #WATER_LAYER).r;

    let f_l = calc_next_flux(cell, vec2<i32>(-1, 0), #WATER_OUTFLOW_L_LAYER, cell_height);
    let f_r = calc_next_flux(cell, vec2<i32>(1, 0), #WATER_OUTFLOW_R_LAYER, cell_height);
    let f_t = calc_next_flux(cell, vec2<i32>(0, -1), #WATER_OUTFLOW_T_LAYER, cell_height);
    let f_b = calc_next_flux(cell, vec2<i32>(0, 1), #WATER_OUTFLOW_B_LAYER, cell_height);
    let f_total = f_l + f_r + f_t + f_b;

    var K = max(1., water / (f_total * d_t));
    if (f_total == 0.) {
        K = 1.;
    }

    let f_l = K * f_l;
    let f_r = K * f_r;
    let f_t = K * f_t;
    let f_b = K * f_b;

    textureStore(heightmap, cell, #WATER_OUTFLOW_L_LAYER, vec4<f32>(f_l, 0., 0., 0.));
    textureStore(heightmap, cell, #WATER_OUTFLOW_R_LAYER, vec4<f32>(f_r, 0., 0., 0.));
    textureStore(heightmap, cell, #WATER_OUTFLOW_T_LAYER, vec4<f32>(f_t, 0., 0., 0.));
    textureStore(heightmap, cell, #WATER_OUTFLOW_B_LAYER, vec4<f32>(f_b, 0., 0., 0.));

}

@compute
@workgroup_size(32)
fn update_water(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell = vec2<i32>(global_id.xy);
    var water: f32 = textureLoad(heightmap, cell, #WATER_LAYER).r;

    let f_l_to_r = textureLoad(heightmap, cell + vec2<i32>(-1, 0), #WATER_OUTFLOW_R_LAYER).r;
    let f_r_to_l = textureLoad(heightmap, cell + vec2<i32>(1, 0), #WATER_OUTFLOW_L_LAYER).r;
    let f_t_to_b = textureLoad(heightmap, cell + vec2<i32>(0, -1), #WATER_OUTFLOW_B_LAYER).r;
    let f_b_to_t = textureLoad(heightmap, cell + vec2<i32>(0, 1), #WATER_OUTFLOW_T_LAYER).r;

    let f_l = textureLoad(heightmap, cell, #WATER_OUTFLOW_L_LAYER).r;
    let f_r = textureLoad(heightmap, cell, #WATER_OUTFLOW_R_LAYER).r;
    let f_b = textureLoad(heightmap, cell, #WATER_OUTFLOW_B_LAYER).r;
    let f_t = textureLoad(heightmap, cell, #WATER_OUTFLOW_T_LAYER).r;

    let water_change = d_t * (
        f_l_to_r +
        f_r_to_l +
        f_t_to_b +
        f_b_to_t - (
            f_l +
            f_r +
            f_t +
            f_b
        )
    );

    water = max(0., water + water_change);

    let evaporation = 0.015;
    water = water * (1. - evaporation * d_t);

    textureStore(heightmap, cell, #WATER_LAYER, vec4<f32>(water, 0., 0., 0.));

    let w_x = 0.5 * (f_l_to_r - f_l + f_r - f_r_to_l);
    let w_y = 0.5 * (f_t_to_b - f_t + f_b - f_b_to_t);

    textureStore(heightmap, cell, #WATER_VELOCITY_X_LAYER, vec4<f32>(w_x, 0., 0., 0.));
    textureStore(heightmap, cell, #WATER_VELOCITY_Y_LAYER, vec4<f32>(w_y, 0., 0., 0.));
}


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

fn get_water_velocity(cell: vec2<i32>) -> vec2<f32> {
    return vec2<f32>(
        textureLoad(heightmap, cell, #WATER_VELOCITY_X_LAYER).r,
        textureLoad(heightmap, cell, #WATER_VELOCITY_Y_LAYER).r
    );
}

@compute
@workgroup_size(32)
fn water_erosion(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell = vec2<i32>(global_id.xy);
    var rock: f32 = textureLoad(heightmap, cell, #ROCK_LAYER).r;
    var soil: f32 = textureLoad(heightmap, cell, #SOIL_LAYER).r;
    var water: f32 = textureLoad(heightmap, cell, #WATER_LAYER).r;
    var sediment: f32 = textureLoad(heightmap, cell, #SEDIMENT_LAYER).r;

    let water_velocity = get_water_velocity(cell);

    let K_c = 1.;
    let carying_capacity = K_c * (1. - get_normal(cell).z) * length(water_velocity);
    // let carying_capacity = 0.1;


    if (sediment < carying_capacity) {
        let rock_hardness = 0.01;
        let rock_to_sediment = d_t * rock_hardness * (carying_capacity - sediment);
        rock = rock - rock_to_sediment;
        sediment = sediment + rock_to_sediment;
    } else {
        let deposit_speed = 1.;
        let deposit = d_t * deposit_speed * (sediment - carying_capacity);
        rock = rock + deposit;
        sediment = sediment - deposit;
    }

    textureStore(heightmap, cell, #ROCK_LAYER, vec4<f32>(rock, 0., 0., 0.));
    textureStore(heightmap, cell, #SEDIMENT_LAYER, vec4<f32>(sediment, 0., 0., 0.));
}


fn bilinear_sample(pos: vec2<f32>, layer: i32) -> f32 {
    let coord = vec2<i32>(pos);
    let p = pos - floor(pos);

    let height_nw = textureLoad(heightmap, vec2<i32>(coord.x, coord.y), layer).r;
    let height_ne = textureLoad(heightmap, vec2<i32>(coord.x + 1, coord.y), layer).r;
    let height_sw = textureLoad(heightmap, vec2<i32>(coord.x, coord.y + 1), layer).r;
    let height_se = textureLoad(heightmap, vec2<i32>(coord.x + 1, coord.y + 1), layer).r;

    return height_nw * (1. - p.x) * (1. - p.y) + height_ne * p.x * (1. - p.y) + height_sw * (1. - p.x) * p.y + height_se * p.x * p.y;
}


@compute
@workgroup_size(32)
fn sediment_movement(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell = vec2<i32>(global_id.xy);
    let water_velocity = get_water_velocity(cell);
    let p = vec2<f32>(cell) - water_velocity;

    let sediment = bilinear_sample(vec2<f32>(cell) - d_t * water_velocity, #SEDIMENT_LAYER);
    textureStore(heightmap, cell, #SEDIMENT_LAYER, vec4<f32>(sediment, 0., 0., 0.));
}



@compute
@workgroup_size(32)
fn thermal_erosion(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell = vec2<i32>(global_id.xy);

    var dir: vec2<i32>;
    if (params.frame % 4 == 0) {
        dir = vec2<i32>(-1, 0);
    } else if (params.frame % 4 == 1) {
        dir = vec2<i32>(1, 0);
    } else if (params.frame % 4 == 2) {
        dir = vec2<i32>(0, -1);
    } else if (params.frame % 4 == 3) {
        dir = vec2<i32>(0, 1);
    }

    var rock = textureLoad(heightmap, cell, #ROCK_LAYER).r;
    let rock_neighbor = textureLoad(heightmap, cell + dir, #ROCK_LAYER).r;

    let d = rock - rock_neighbor;
    if (abs(d) > 0.1) {
        rock = rock - d / 4.;
    }

    textureStore(heightmap, cell, #ROCK_LAYER, vec4<f32>(rock, 0., 0., 0.));
}
