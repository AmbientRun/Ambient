const atmo_radius = 6471e3;
const planet_radius = 6371e3;

struct Node {
    density: f32,
    origin: vec3<f32>,
    size: f32,
    children: array<u32, 8>,
};

struct CloudBuffer {
    clouds: array<Node>,
};

@group(#MATERIAL_BIND_GROUP)
@binding(0)
var<storage> cloud_buffer: CloudBuffer;

struct CloudBuffer {
    clouds: array<Node>,
};

fn sphere_intersect(pos: vec3<f32>, dir: vec3<f32>, r: f32) -> vec2<f32> {
    let a = dot(dir, dir);
    let b = 2.0 * dot(dir, pos);
    let c = dot(pos, pos) - (r * r);
    let d = (b * b) - 4.0 * a * c;

    if (d < 0.) {
        return vec2<f32>(1e5, -1e5);
    }

    return vec2<f32>(
        max((-b - sqrt(d)) / (2.0 * a), 0.0),
        (-b + sqrt(d)) / (2.0 * a),
    );
}

const DISTANCE_THRESHOLD: f32 = 0.1;

fn cube_intersect(origin: f32, size: f32, ray_o: vec3<f32>, dir: vec3<f32>) ->
f32 {
    let origin = ray_o - origin;
    let inv_dir = vec3<f32>(1. / dir.x, 1. / dir.y, 1. / dir.z);
    let t1 = (-size - origin) * inv_dir;
    let t2 = (size - origin) * inv_dir;

    let tmin = min(t1, t2);
    let tmax = max(t1, t2);

    let tmin = max(tmin.x, max(tmin.y, tmin.z));
    let tmax = min(tmax.x, min(tmax.y, tmax.z));

    return f32(tmax > 0.) * (tmax - tmin);
}

// Returns the distance along the ray, or < 0
fn node_ray(node: Node, origin: vec3<f32>, dir: vec3<f32>) -> f32 {
    return 1.0;
}

fn scattering(depth: f32, pos: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    let orig = pos + vec3<f32>(0.0, 0.0, planet_radius);
    // Get atmosphere intersection
    let ray_l = sphere_intersect(orig, dir, atmo_radius);

    let dist =  ray_l.y - ray_l.x;

    let dir = normalize(dir);

    let steps = 16;
    let step_len = dist / f32(steps);

    let light_dir = global_params.sun_direction.xyz;
    let u = dot(dir, light_dir);
    let g = 0.76;
    let uu = u*u;
    let gg = g*g;

    let beta_ray = vec3<f32>(5.5e-6, 13.0e-6, 22.4e-6);
    // let beta_ray = vec3<f32>(3.8e-6, 5.5e-7, 16.1e-6);
    let beta_mie = vec3<f32>(21e-6);

    let allow_mie = depth >= ray_l.y || depth > global_params.camera_far * 0.9;
    // How likely is light from the sun to scatter to us
    let phase_ray = max(3.0 / (16.0*PI) * (1.0 + uu), 0.);
    // 3 / (16pi) * cos2()
    let phase_mie = max((3.0 / (8.0 * PI)) * ((1.0 - gg) * (1.0 + uu)) / ((2.0 +
    gg) * pow(1.0 + gg - 2.0*g*u, 1.5)), 0.);

    let phase_mie = phase_mie * f32(allow_mie);

    // Accumulation of scattered light
    var total_ray = vec3<f32>(0.);
    var total_mie = vec3<f32>(0.);

    // Optical depth
    var rayleigh = 0.0;
    var mie = 0.0;

    let Hr = 8e3;
    let Hm = 1.2e3;

    var pos_i = 0.0;

    // Primary ray
    for (var i = 0; i < steps; i = i + 1) {
        let p = orig + dir * pos_i;

        let height = length(p) - planet_radius;
        let hr = exp(-height / Hr) * step_len;
        let hm = exp(-height / Hm) * step_len;

        // Accumulate density along viewing ray
        rayleigh = rayleigh + max(hr, 0.);
        mie = mie + max(hm, 0.);

        // Distance from ray sample to the atmosphere towards the sun
        let ray = sphere_intersect(p, light_dir, atmo_radius);

        // if (dist < 0.0) { return vec3<f32>(1., 0., 0.); }

        // Cast ray into the sun
        let sun_steps = 8;
        let sun_len = ray.y / f32(sun_steps);
        // Density along light ray
        var l_r = 0.0;
        var l_m = 0.0;

        var pos_l = ray_l.x;

        for (var j = 0; j < sun_steps; j = j + 1) {
            // let l_pos = p + light_dir * f32(j) * sun_len;
            let p = p + light_dir * pos_l;

            let height_l = length(p) - planet_radius;
            let h_ray = exp(-height_l / Hr) * sun_len;
            let h_mie = exp(-height_l / Hm) * sun_len;

            l_r = l_r + max(h_ray, 0.);
            l_m = l_m + max(h_mie, 0.);

            pos_l = pos_l + sun_len;
        }

        // Add the results of light integration by using the accumulated density
        // and beta coeff
        let tau =
            beta_ray * (rayleigh + l_r)
            + beta_mie * (mie + l_m);

        let attn = exp(-tau);

        total_ray = total_ray + attn * hr;
        total_mie = total_mie + attn * hm;

        // Travel forward
        pos_i = pos_i + step_len;
    }

    let result =
    (
          total_ray * beta_ray * phase_ray
        + total_mie * beta_mie * phase_mie
    ) * 20.0;

    return result;
}

fn get_sky_color(
    depth: f32,
    origin: vec3<f32>,
    forward: vec3<f32>,
) -> vec3<f32> {
    let spot_rad = 1.0 - dot(forward, global_params.sun_direction.xyz);
    let d = 2000.0;
    let g = 3.0;
    let spot = exp(-pow(d * spot_rad, g));


    return scattering(depth, origin, forward)
    + global_params.sun_diffuse.rgb * spot;
}
