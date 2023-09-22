
struct Node {
    density: f32,
    origin: vec3<f32>,
    size: f32,
    children: array<u32, 8>,
};

struct CloudBuffer {
    clouds: array<Node>,
};

@group(MATERIAL_BIND_GROUP)
@binding(0)
var<storage> cloud_buffer: CloudBuffer;

fn sphere_intersect(pos: vec3<f32>, dir: vec3<f32>, r: f32) -> vec2<f32> {
    let a = dot(dir, dir);
    let b = 2.0 * dot(dir, pos);
    let c = dot(pos, pos) - (r * r);
    let d = (b * b) - 4.0 * a * c;

    if d < 0. {
        return vec2<f32>(1e5, -1e5);
    }

    return vec2<f32>(
        max((-b - sqrt(d)) / (2.0 * a), 0.0),
        (-b + sqrt(d)) / (2.0 * a),
    );
}

const DISTANCE_THRESHOLD: f32 = 0.1;

fn cube_intersect(orig: f32, size: f32, ray_o: vec3<f32>, dir: vec3<f32>) -> f32 {
    let origin = ray_o - orig;
    let inv_dir = vec3<f32>(1. / dir.x, 1. / dir.y, 1. / dir.z);
    let t1 = (-size - origin) * inv_dir;
    let t2 = (size - origin) * inv_dir;

    var tmin = min(t1, t2);
    var tmax = max(t1, t2);

    let bestMin = max(tmin.x, max(tmin.y, tmin.z));
    let bestMax = min(tmax.x, min(tmax.y, tmax.z));

    return f32(bestMax > 0.) * (bestMax - bestMin);
}

// Returns the distance along the ray, or < 0
fn node_ray(node: Node, origin: vec3<f32>, dir: vec3<f32>) -> f32 {
    return 1.0;
}

const RAY_BETA: vec3f = vec3f(5.5e-6, 13.0e-6, 22.4e-6);
const MIE_BETA: vec3f = vec3f(21e-6, 21e-6, 21e-6);
const MIE_SCATTER: f32 = 0.7;

const AMBIENT_BETA: vec3f = vec3f(0.0, 0.0, 0.0);
const ABSORPTION_BETA: vec3f = vec3f(2.04e-5, 4.97e-5, 1.95e-6);

const RAY_HEIGHT: f32 = 8e3;
const MIE_HEIGHT: f32 = 1.2e3;

const ABSORPTION_HEIGHT: f32 = 30e3;
const ABSORPTION_FALLOFF: f32 = 4e3;

const ATMO_RADIUS = 6471e3;
const PLANET_RADIUS = 6371e3;

const STEPS_I: u32 = 16u;
const STEPS_L: u32 = 4u;

/// `ray_start` relative to the planet's center
fn scatter(ray_start: vec3f, ray_dir: vec3f, max_dist: f32) -> vec3f {
    /// Ray sphere intersection for the viewing ray
    var a = dot(ray_dir, ray_dir);
    var b = 2.0 * dot(ray_dir, ray_start);
    var c = dot(ray_start, ray_start) - (ATMO_RADIUS * ATMO_RADIUS);
    var d = (b * b) - 4.0 * a * c;

    if d < 0.0 {
        return vec3f(0.0, 0.0, 0.0);
    }

    var ray_len = vec2f(max((-b - sqrt(d)) / (2.0 * a), 0.0), min((-b + sqrt(d)) / (2.0 * a), max_dist));

    if ray_len.x > ray_len.y {
        return vec3f(0.0, 0.0, 0.0);
    }

    /// Disable mi if the atmosphere is occluded
    let allow_mie = max_dist > ray_len.y;

    // Clamp to the atmosphere or occlusion
    ray_len = vec2(max(ray_len.x, 0.0), min(ray_len.y, max_dist));

    let step_size_i = (ray_len.y - ray_len.x) / f32(STEPS_I);
    var ray_pos_i = ray_len.x + step_size_i * 0.5;

    var total_ray = vec3f(0.0);
    var total_mie = vec3f(0.0);

    // Integrated air volume along the ray
    var opt_i = vec3(0.0);

    let scale_height = vec2f(RAY_HEIGHT, MIE_HEIGHT);

    let light_dir = normalize(global_params.sun_direction.xyz);
    // Mie scattering is proportional to the angle between the ray and the sun
    let mu = dot(ray_dir, light_dir);
    let mumu = mu * mu;


    let gg = MIE_SCATTER * MIE_SCATTER;

    let phase_ray = 3.0 / (16.0 * PI) * (1.0 + mumu);

    let phase_mie = f32(allow_mie) * 3.0 / (25.1327412287) * ((1.0 - gg) * (mumu + 1.0)) / (pow(1.0 + gg - 2.0 * mu * MIE_SCATTER, 1.5) * (2.0 + gg));

    for (var i = 0u; i < STEPS_I; i++) {
        let pos_i = ray_start + ray_dir * ray_pos_i;
        let height_i = length(pos_i) - PLANET_RADIUS;

        // Calculate the amount of air inside this step based on height based air density falloff
        var density = vec3f(exp(-height_i / scale_height), 0.0);

        let denom = (ABSORPTION_HEIGHT - height_i) / ABSORPTION_FALLOFF;
        density.z = (1.0 / (denom * denom + 1.0)) * density.x;

        density *= step_size_i;

        opt_i += density;

        // ray-sphere intersect toward the sun from the current primary ray position to the edge of the atmosphere (space)
        a = dot(light_dir, light_dir);
        b = 2.0 * dot(light_dir, pos_i);
        c = dot(pos_i, pos_i) - (ATMO_RADIUS * ATMO_RADIUS);
        d = (b * b) - 4.0 * a * c;

        let step_size_l: f32 = (-b + sqrt(d)) / (2.0 * a * f32(STEPS_L));

        var ray_pos_l = step_size_l * 0.5;

        var opt_l = vec3(0.0);

        for (var l = 0u; l < STEPS_L; l++) {
            let pos_l = pos_i + light_dir * ray_pos_l;

            let height_l = length(pos_l) - PLANET_RADIUS;

            // Do the same for the secondary ray
            var density_l = vec3f(exp(-height_l / scale_height), 0.0);
            var denom = (ABSORPTION_HEIGHT - height_l) / ABSORPTION_FALLOFF;
            density_l.z = (1.0 / (denom * denom + 1.0)) * density_l.x;

            density_l *= step_size_l;

            opt_l += density_l;

            ray_pos_l += step_size_l;
        }

        let attn: vec3f = exp(-RAY_BETA * (opt_i.x + opt_l.x) - MIE_BETA * (opt_i.y + opt_l.y) - ABSORPTION_BETA * (opt_i.z + opt_l.z));

        // accumulate the scattered light (how much will be scattered towards the camera)
        total_ray += density.x * attn;
        total_mie += density.y * attn;

        // and increment the position on this ray
        ray_pos_i += step_size_i;
    }

    // let opacity: vec3f = exp(-(MIE_BETA * opt_i.y + RAY_BETA * opt_i.x + ABSORPTION_BETA * opt_i.z));

    let res_ray: vec3f = vec3f(phase_ray * RAY_BETA * total_ray);
    let res_mie: vec3f = vec3f(phase_mie * MIE_BETA * total_mie);
    let res_amb: vec3f = vec3f(opt_i.x * AMBIENT_BETA);

    let res = res_ray + res_mie + res_amb;
    return res;
}

const SUN_ANG_INNER: f32 = 0.9996;
const SUN_ANG_OUTER: f32 = 0.999;

/// Depth ranges from 0..=1
fn get_sky_color(
    depth: f32,
    origin: vec3<f32>,
    forward: vec3<f32>,
) -> vec3<f32> {

    let spot_rad = 1.0 - dot(forward, normalize(global_params.sun_direction.xyz));
    let d = 2000.0;
    let spot = exp(-pow(d * spot_rad, 3.0));

    var max_dist = depth * global_params.camera_far;

    if depth > 0.9 {
        max_dist = 1e12;
    }

    let color = (scatter(origin + vec3<f32>(0.0, 0.0, PLANET_RADIUS + 100.0), normalize(forward), max_dist) + spot) * 40.0 * global_params.sun_diffuse.rgb;

    return color;
}
