struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

struct Scene {
    color: vec3<f32>,
};

struct Conf {
    steps_i: i32,
    steps_l: i32,
};

struct Planet {
    pos: vec3<f32>,
    radius: f32,
    // Radius of atmosphere from planet core
    atmo_radius: f32,

};

fn camera_ray(res: vec3<f32>, coord: vec2<f32>) -> vec3<f32> {
    let uv = coord.xy - vec2<f32>(0.5);
    return normalize(vec3<f32>(uv.x, uv.y, -1.0));
}

struct Volume {
    absorption: vec3<f32>, // How much light passes through

};

struct Sample {
    dist: f32,
    vol: Volume,
};

const air_volume   = Volume(vec3<f32>(1e-6, 1e-6, 1e-6));
const cloud_volume = Volume(vec3<f32>(0.1, 0.1, 0.1));

fn smooth_min(a: Sample, b: Sample, t: f32) -> Sample {
    let h = max(t - abs(a.dist - b.dist), 0.0);
    var vol = a.vol;
    if b.dist < 0.0 {
        vol = b.vol;
    }

    return Sample(min(a.dist, b.dist) - h * h * h / (6.0 * t * t), vol);
}

fn sdf(p: vec3<f32>) -> Sample {
    var min_d = Sample(10000.0, air_volume);
    // for (var i = 0; i < params.count; i = i + 1) {
    //     let dist = cloud_sdf(cloud_buffer.clouds[i], p);

    //     min_d = smooth_min(min_d, dist, 2.0);
    // }

    return min_d;
}

// fn calc_normal(pos: vec3<f32>) -> vec3<f32> {
//     // Center sample
//     let c = sdf(pos);
//     // Use offset samples to compute gradient / normal
//     let eps_zero = vec2<f32>(0.001, 0.0);
//     return normalize(vec3<f32>(sdf(pos + eps_zero.xyy), sdf(pos + eps_zero.yxy), sdf(pos + eps_zero.yyx)) - c);
// }

// fn raymarch(uv: vec2<f32>, origin: vec3<f32>, dir: vec3<f32>, scene: Scene, planet: Planet) -> vec4<f32> {
//     let max_steps = 100;
//     let eps = 0.01;
//     let close = 0.1;
//     let max_d = 1000.0;

//     var depth = 0.0;

//     var dens = 0.0;

//     for (var i = 0; i < max_steps; i = i + 1) {
//         let p = origin + depth * dir;
//         let dist = sdf(p);
//         let abs_dist = abs(dist);

//         // Boundary
//         if (dist < close) {
//             depth = depth + 0.1;
//         }

//         if (dist < 0.0) {
//             dens = dens + abs_dist;
//         }

//         if (depth > max_d) {
//             break;
//         }

//         depth = depth + abs_dist;
//     }

//     let opacity = exp(-dens);

//     // let sky = get_sky_color(uv, dir, 1e6, scene, planet);
//     let sky = vec4<f32>(0.,0.,1., 1.);

//     let cloud = vec4<f32>(1.,1.,1.,1.);
//     return mix(cloud, sky, opacity);
// }


@vertex
fn vs_main(@builtin(instance_index) instance_index: u32, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {

    var out: VertexOutput;
    let x = i32(vertex_index) / 2;
    let y = i32(vertex_index) & 1;
    let tc = vec2<f32>(
        f32(x) * 2.0,
        f32(y) * 2.0
    );
    out.position = vec4<f32>(
        tc.x * 2.0 - 1.0,
        1.0 - tc.y * 2.0,
        0.000001,
        1.0
    );
    out.world_position = global_params.inv_projection_view * out.position;
    out.world_position = out.world_position / out.world_position.w;
    out.uv = tc;
    return out;
}

@fragment
fn fs_forward_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene = Scene(vec3<f32>(0., 0., 0.));
    let planet = Planet(vec3<f32>(0.0, 0.0, 0.0), 6371e3, 6471e3);
    let dir = normalize(in.world_position.xyz - global_params.camera_position.xyz);

    // let color = get_sky_color(in.uv, global_params.camera_position.xyz, dir, scene, planet);
    let depth = (1. - textureSampleLevel(solids_screen_depth, default_sampler, in.uv, 0.)) * global_params.camera_far;
    var color = get_sky_color(depth, global_params.camera_position.xyz, dir);
    color = 1.0 - exp(-color);

    return vec4<f32>(apply_fog(color, global_params.camera_position.xyz, in.world_position.xyz), 1.0);
}
