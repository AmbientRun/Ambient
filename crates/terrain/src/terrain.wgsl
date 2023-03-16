

struct TerrainMaterialParams {
    heightmap_position: vec3<f32>,
    lod_factor: f32,
    cell_diagonal: f32,
};
@group(#MATERIAL_BIND_GROUP)
@binding(0)
var<uniform> terrain_params: TerrainMaterialParams;

@group(#MATERIAL_BIND_GROUP)
@binding(1)
var heightmap_sampler: sampler;

@group(#MATERIAL_BIND_GROUP)
@binding(2)
var heightmap: texture_2d_array<f32>;

@group(#MATERIAL_BIND_GROUP)
@binding(3)
var normalmap: texture_2d<f32>;

@group(#MATERIAL_BIND_GROUP)
@binding(4)
var surface_color_2k: texture_2d_array<f32>;

@group(#MATERIAL_BIND_GROUP)
@binding(5)
var surface_normals_2k: texture_2d_array<f32>;

@group(#MATERIAL_BIND_GROUP)
@binding(6)
var texture_sampler: sampler;


struct TerrainTriplanarSettings {
    top_color: vec3<f32>,
    rot_top_45_deg: u32,
    side_color: vec3<f32>,
    rot_side_45_deg: u32,
    top_scale: f32,
    side_scale: f32,
    hardness: f32,
    angle: f32,
};
struct TerrainTriplanarSample {
    settings: TerrainTriplanarSettings,
    top_texture: i32,
    side_texture: i32,
};
struct TerrainMaterialSettings {
    grass_depth: f32,
    grass_gradient: f32,
    rock_soil_gradient: f32,
    beach_gradient: f32,
    beach_noise_scale: f32,
    beach_noise_steepness: f32,
    variation_texture_scale: f32,
    variation_gradient: f32,
};
struct TerrainMaterialDef {
    soft_rock1: TerrainTriplanarSample,
    soft_rock2: TerrainTriplanarSample,
    hard_rock1: TerrainTriplanarSample,
    hard_rock2: TerrainTriplanarSample,
    forest_floor1: TerrainTriplanarSample,
    forest_floor2: TerrainTriplanarSample,
    grass1: TerrainTriplanarSample,
    grass2: TerrainTriplanarSample,
    sand: TerrainTriplanarSample,

    settings: TerrainMaterialSettings,
};
@group(#MATERIAL_BIND_GROUP)
@binding(7)
var<uniform> terrain_mat_def: TerrainMaterialDef;

@group(#MATERIAL_BIND_GROUP)
@binding(8)
var noise_texture: texture_2d<f32>;


struct VertexOutput {
    @location(0) texcoord: vec2<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) instance_index: u32,
    @builtin(position) position: vec4<f32>,
};


fn get_entity_primitive_mesh(loc: vec2<u32>, index: u32) -> u32 {
    let i = index >> 2u;
    let j = index & 3u;

    var meshes = get_entity_gpu_primitives_mesh(loc);
    return bitcast<u32>(meshes[i][j]);
}

@vertex
fn vs_main(@builtin(instance_index) instance_index: u32, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let primitive = primitives.data[instance_index];
    let entity_loc = primitive.xy;
    let mesh_index = get_entity_primitive_mesh(entity_loc, primitive.z);

    let heightmap_size = vec2<f32>(textureDimensions(heightmap));

    let local_to_world = get_entity_mesh_to_world(entity_loc);

    let world_position = local_to_world * vec4<f32>(get_mesh_position(mesh_index, vertex_index), 1.);
    let world_position_norm = world_position / world_position.w;

    let dist_from_camera = length(world_position_norm.xyz - global_params.forward_camera_position.xyz);
    let lod = floor(sqrt(dist_from_camera) * terrain_params.lod_factor);
    let lod_start = pow(lod / terrain_params.lod_factor, 2.);
    let lod_blended = interpolate_clamped_1_1(dist_from_camera, lod_start, lod_start + terrain_params.cell_diagonal, max(lod - 1., 0.), lod);

    // The aligned_xy tells us where this vertex would be on a lower lod level mesh
    let aligned_xy = floor(world_position_norm.xy / pow(2., lod)) * pow(2., lod);
    let blended_xy = interpolate_clamped_1_2(dist_from_camera, lod_start, lod_start + terrain_params.cell_diagonal, world_position_norm.xy, aligned_xy);

    // This is only "exact" (texel -> vertex) on lod0, on higher lods it samples between texels
    out.texcoord = (blended_xy - terrain_params.heightmap_position.xy + 0.5) / heightmap_size;
    var height = 0.;
    for (var i = 0; i < 2; i = i + 1) {
        height = height + textureSampleLevel(heightmap, heightmap_sampler, out.texcoord, i, 0.).r;
    }

    out.world_position = vec4<f32>(blended_xy, height + f32(#TERRAIN_BASE), 1.);
    out.position = global_params.projection_view * out.world_position;
    out.instance_index = instance_index;

    return out;
}


@fragment
fn fs_shadow_main(in: VertexOutput) {
}

fn maybe_rot_45deg(v: vec2<f32>, rot: bool) -> vec2<f32> {
    if rot {
        return vec2<f32>(
            // (v.x - v.y) / sqrt(2.),
            // (v.y + v.x) / sqrt(2.)
            (v.x + v.y) / sqrt(2.),
            (v.y - v.x) / sqrt(2.)
        );
    } else {
        return v;
    }
}

struct TextureSample {
    color: vec3<f32>,
    normal: vec3<f32>,
};
fn mix_sample(a: TextureSample, b: TextureSample, p: f32) -> TextureSample {
    var res: TextureSample;
    res.color = mix(a.color, b.color, p);
    res.normal = mix(a.normal, b.normal, p);
    // res.normal = normalize(vec3<f32>(
    //     a.normal.xy * b.normal.z + b.normal.xy * a.normal.z,
    //     a.normal.z * b.normal.z
    // ));
    return res;
}

fn comp_normalize(v: vec3<f32>) -> vec3<f32> {
    return v / (v.x + v.y + v.z);
}

fn triplanar_sample(p: vec3<f32>, normal: vec3<f32>, terrain_sample: TerrainTriplanarSample) -> TextureSample {
    let settings = terrain_sample.settings;

    let normal = comp_normalize(pow(abs(normal), vec3<f32>(1., 1., settings.angle) * settings.hardness));

    let z_tc = maybe_rot_45deg(p.xy / settings.top_scale, bool(settings.rot_side_45_deg));
    let x_tc = maybe_rot_45deg(vec2<f32>(p.y, -p.z) / settings.side_scale, bool(settings.rot_side_45_deg));
    let y_tc = maybe_rot_45deg(-p.xz / settings.side_scale, bool(settings.rot_side_45_deg));
    let z_color = textureSample(surface_color_2k, texture_sampler, z_tc, terrain_sample.top_texture).rgb * settings.top_color;
    let x_color = textureSample(surface_color_2k, texture_sampler, x_tc, terrain_sample.side_texture).rgb * settings.side_color;
    let y_color = textureSample(surface_color_2k, texture_sampler, y_tc, terrain_sample.side_texture).rgb * settings.side_color;

    // let z_normal = textureSample(surface_normals_2k, texture_sampler, z_tc, top_layer).rgb;
    // let x_normal = textureSample(surface_normals_2k, texture_sampler, x_tc, side_layer).rgb;
    // let y_normal = textureSample(surface_normals_2k, texture_sampler, y_tc, side_layer).rgb;
    var res: TextureSample;
    res.color = normal.x * x_color + normal.y * y_color + normal.z * z_color;
    // res.color = vec3<f32>(0.1, 0.1, 0.1);
    // res.normal = normal.x * x_normal + normal.y * y_normal + normal.z * z_normal;
    // res.normal = normalize(res.normal * 2. - 1.);
    return res;
}

#TERRAIN_FUNCS

fn get_hardness_sampled(tc: vec2<f32>, height: f32) -> f32 {
    let hardness = textureSampleLevel(heightmap, heightmap_sampler, tc, # HARDNESS_LAYER, 0.).r;
    let amount = textureSampleLevel(heightmap, heightmap_sampler, tc, # HARDNESS_STRATA_AMOUNT_LAYER, 0.).r;
    let wavelength = textureSampleLevel(heightmap, heightmap_sampler, tc, # HARDNESS_STRATA_WAVELENGTH_LAYER, 0.).r;
    return hardness_calc(hardness, amount, wavelength, height);
}

@fragment
fn fs_forward_main(in: VertexOutput) -> MainFsOut {
    var material: MaterialOutput;

    material.emissive_factor = vec3<f32>(0., 0., 0.);
    material.alpha_cutoff = 0.5;
    material.opacity = 1.;
    material.shading = 1.;
    material.roughness = 0.8;
    material.metallic = 0.;

    let normal = textureSample(normalmap, heightmap_sampler, in.texcoord).xyz;
    material.normal = normal;
    let tangent = cross(normal, vec3<f32>(0., 1., 0.));
    let bitangent = cross(normal, tangent);
    let normal_mat = mat3x3<f32>(tangent, bitangent, normal);

    let soil_amount = textureSampleLevel(heightmap, heightmap_sampler, in.texcoord, # SOIL_LAYER, 0.).r;
    let texture_variation = textureSample(noise_texture, texture_sampler, in.world_position.xy / terrain_mat_def.settings.variation_texture_scale).r;
    let hardness = get_hardness_sampled(in.texcoord, in.world_position.z);
    let texture_variation = interpolate_clamped_1_1(texture_variation, 0.5 - terrain_mat_def.settings.variation_gradient / 2., 0.5 + terrain_mat_def.settings.variation_gradient / 2., 0., 1.);

    let height_over_ocean = in.world_position.z;
    let beach_amount = smoothstep(5., 1., height_over_ocean);
    let beach_wetness = mix(0.5, 1., smoothstep(0., 3., height_over_ocean));

    let soft_rock1 = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.soft_rock1);
    let soft_rock2 = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.soft_rock2);

    let hard_rock1 = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.hard_rock1);
    let hard_rock2 = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.hard_rock2);

    var forest_floor1 = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.forest_floor1);
    var forest_floor2 = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.forest_floor2);

    var grass1 = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.grass1);
    var grass2 = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.grass2);

    var sand = triplanar_sample(in.world_position.xyz, normal, terrain_mat_def.sand);
    sand.color = sand.color * beach_wetness;

    let soft_rock = mix_sample(soft_rock1, soft_rock2, texture_variation);
    let hard_rock = mix_sample(hard_rock1, hard_rock2, texture_variation);
    let rock = mix_sample(soft_rock, hard_rock, hardness);
    var forest_floor = mix_sample(forest_floor1, forest_floor2, texture_variation);
    var grass = mix_sample(grass1, grass2, texture_variation);
    var soil = mix_sample(grass, forest_floor, interpolate_clamped_1_1(soil_amount, terrain_mat_def.settings.grass_depth - terrain_mat_def.settings.grass_gradient, terrain_mat_def.settings.grass_depth + terrain_mat_def.settings.grass_gradient, 0., 1.));

    let rock_soil = mix_sample(rock, soil, interpolate_clamped_1_1(soil_amount, 0., terrain_mat_def.settings.rock_soil_gradient, 0., 1.));


    let beach_noise = textureSample(noise_texture, texture_sampler, in.world_position.xy / terrain_mat_def.settings.beach_noise_scale).r;
    let rock_soil_sand = mix_sample(sand, rock_soil, smoothstep(beach_amount - terrain_mat_def.settings.beach_gradient, beach_amount, pow(beach_noise, terrain_mat_def.settings.beach_noise_steepness)));
    material.base_color = rock_soil_sand.color;

    // let z_color = textureSample(surface_color_2k, texture_sampler, in.world_position.xy / 2., 0).rgb;
    // material.base_color = z_color;

    // let water = textureSampleLevel(heightmap, heightmap_sampler, in.texcoord, #WATER_LAYER, 0.).r;
    // material.base_color = mix(material.base_color, vec3<f32>(0., 0., 1.), water * 0.1);

    let entity_color = get_entity_color_or(primitives.data[in.instance_index].xy, vec4<f32>(1., 1., 1., 1.));
    material.base_color = material.base_color * entity_color.rgb;
    material.opacity = entity_color.a;
    // material.base_color = vec3<f32>(texture_variation);

    // let local_normal = normal_mat * rock_soil_sand.normal;

    let x = mat3_from_quat(quat_from_mat3(normal_mat)) * vec3<f32>(0., 0., 1.);

    return MainFsOut(
        shading(material, in.world_position),
        quat_from_mat3(normal_mat)
    );
}

@fragment
fn fs_outlines_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let entity_loc = primitives.data[in.instance_index].xy;
    return get_entity_outline_or(entity_loc, vec4<f32>(0., 0., 0., 0.));
}
