
@group(0)
@binding(0)
var heightmap: texture_2d_array<f32>;

@group(0)
@binding(1)
var normalmap: texture_2d<f32>;

@group(0)
@binding(2)
var heightmap_sampler: sampler;

struct NaturalEntity {
    position: vec3<f32>,
    scale: f32,
    rotation: vec3<f32>,
    element: u32,
};
struct EntitiesBuffer { data: array<NaturalEntity>, };
@group(0)
@binding(3)
var<storage, read_write> output_entities: EntitiesBuffer;

struct AtomicU32Buffer { data: atomic<u32>, };
@group(0)
@binding(4)
var<storage, read_write> output_counts: AtomicU32Buffer;

@group(0)
@binding(5)
var blue_noise: texture_2d<f32>;

@group(0)
@binding(6)
var cluster_noise: texture_2d<f32>;

struct NaturalCurve {
    params: vec4<f32>,
    kind: u32,
};

struct NaturalElement {
    soil_depth: NaturalCurve,
    elevation: NaturalCurve,
    water_depth: NaturalCurve,
    steepness: NaturalCurve,
    cluster_noise: NaturalCurve,
    scale_min: f32,
    scale_max: f32,
    scale_power: f32,
    rotation_x: f32,
    rotation_y: f32,
    rotation_z: f32,
    rotation_z_jitter: f32,
    rotation_xy_jitter: f32,
    rotation_straightness: f32,
    position_normal_offset: f32,
    position_z_offset: f32,
    normal_miplevel: f32,
    cluster_noise_scale: f32,
};

struct NaturalElements { data: array<NaturalElement>, };
@group(0)
@binding(7)
var<storage> natural_elements: NaturalElements;

@group(0)
@binding(8)
var default_sampler: sampler;

fn sample_curve(curve: NaturalCurve, x: f32) -> f32 {
    if (curve.kind == 0u) {
        return curve.params.x;
    } else if (curve.kind == 1u) {
        return interpolate_1_1(x, curve.params.x, curve.params.y, curve.params.z, curve.params.w);
    } else if (curve.kind == 2u) {
        return interpolate_clamped_1_1(x, curve.params.x, curve.params.y, curve.params.z, curve.params.w);
    } else if (curve.kind == 3u) {
        return mix(curve.params.z, curve.params.w, smoothstep(curve.params.x, curve.params.y, x));
    } else if (curve.kind == 4u) {
        // Gaussian function
        let center = curve.params.x;
        let std_dev = curve.params.y;
        let k = x - center;
        return mix(curve.params.z, curve.params.w, exp(-k * k / (2. * std_dev * std_dev)));
    } else {
        return 0.;
    }
}

fn cdf_sample(probabilities: ptr<function, array<f32, 200>>, length: u32, r: f32) -> u32 {
    var total_probability: f32 = 0.;
    for (var i=0u; i < length; i = i + 1u) {
        total_probability = total_probability + (*probabilities)[i];
    }
    var culm: f32 = 0.;
    for (var i=0u; i < length; i = i + 1u) {
        culm = culm + (*probabilities)[i] / total_probability;
        if (r <= culm) {
            return i;
        }
    }
    return 0u;
}

fn rad_to_deg(rad: f32) -> f32 {
    return rad * 180. / 3.14159;
}

@compute
@workgroup_size(#WORKGROUP_SIZE)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) workgroups: vec3<u32>) {
    let heightmap_size = vec2<f32>(textureDimensions(heightmap));
    let grid_size = heightmap_size / vec2<f32>(f32(workgroups.x * u32(#WORKGROUP_SIZE)), f32(workgroups.y));

    let px = textureLoad(blue_noise, vec2<i32>(id.xy + 20u) % 64, 0).r;
    let py = textureLoad(blue_noise, vec2<i32>(id.xy + 25u) % 64, 0).r;
    let position = (vec2<f32>(id.xy) + vec2<f32>(px, py)) * grid_size;

    let tc = (position + 0.5) / heightmap_size;

    let rock = textureSampleLevel(heightmap, heightmap_sampler, tc, #ROCK_LAYER, 0.).r;
    let soil = textureSampleLevel(heightmap, heightmap_sampler, tc, #SOIL_LAYER, 0.).r;
    let elevation = rock + soil;
    let height_over_ocean = elevation + f32(#TERRAIN_BASE);

    var element_probability: array<f32, 200>;
    for (var i=0u; i < arrayLength(&natural_elements.data); i = i + 1u) {
        let element_conf = natural_elements.data[i];

        let normal = textureSampleLevel(normalmap, heightmap_sampler, tc, element_conf.normal_miplevel).xyz;
        let steepness = rad_to_deg(acos(normal.z));

        let cluster_noise_offset = vec2<f32>(f32(i % 7u) / 7., f32(i % 5u) / 5.);
        let cluster_noise = textureSampleLevel(cluster_noise, default_sampler, tc / element_conf.cluster_noise_scale + cluster_noise_offset, 0.).r;

        element_probability[i] =
            clamp(sample_curve(element_conf.soil_depth, soil), 0., 1.) *
            clamp(sample_curve(element_conf.elevation, height_over_ocean), 0., 1.) *
            clamp(sample_curve(element_conf.water_depth, max(0., -height_over_ocean)), 0., 1.) *
            clamp(sample_curve(element_conf.steepness, steepness), 0., 1.) *
            clamp(sample_curve(element_conf.cluster_noise, cluster_noise), 0., 1.);
    }
    let r = textureLoad(blue_noise, vec2<i32>(id.xy + 30u) % 64, 0).r;
    let element = cdf_sample(&element_probability, arrayLength(&natural_elements.data), r);

    let threashold = textureLoad(blue_noise, vec2<i32>(id.xy) % 64, 0).r;
    if (element_probability[element] > threashold) {
        let element_conf = natural_elements.data[element];
        let out_offset = atomicAdd(&output_counts.data, 1u);

        let normal = textureSampleLevel(normalmap, heightmap_sampler, tc, element_conf.normal_miplevel).xyz;

        output_entities.data[out_offset].position = vec3<f32>(position, elevation) +
            normal * element_conf.position_normal_offset +
            vec3<f32>(0., 0., element_conf.position_z_offset);

        let rot_z_jitter = textureLoad(blue_noise, vec2<i32>(id.xy + 36u) % 64, 0).r;
        let rot_x_jitter = textureLoad(blue_noise, vec2<i32>(id.xy + vec2<u32>(12u, 36u)) % 64, 0).r;
        let rot_y_jitter = textureLoad(blue_noise, vec2<i32>(id.xy + vec2<u32>(36u, 12u)) % 64, 0).r;
        output_entities.data[out_offset].rotation.z = atan2(normal.y, normal.x) +
            (2. * rot_z_jitter - 1.) * 3.14159 * element_conf.rotation_z_jitter +
            element_conf.rotation_z;
        output_entities.data[out_offset].rotation.x = -asin(normal.y) * (1. - element_conf.rotation_straightness) +
            (2. * rot_x_jitter - 1.) * 3.14159 * element_conf.rotation_xy_jitter +
            element_conf.rotation_x;
        output_entities.data[out_offset].rotation.y = asin(normal.x) * (1. - element_conf.rotation_straightness) +
            (2. * rot_y_jitter - 1.) * 3.14159 * element_conf.rotation_xy_jitter +
            element_conf.rotation_y;

        let scale = textureLoad(blue_noise, vec2<i32>(id.xy + 40u) % 64, 0).r;
        output_entities.data[out_offset].scale = mix(
            element_conf.scale_min,
            element_conf.scale_max,
            pow(scale, element_conf.scale_power)
        );
        output_entities.data[out_offset].element = element;
    }
}
