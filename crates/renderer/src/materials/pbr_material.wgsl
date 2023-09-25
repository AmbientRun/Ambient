struct PbrMaterialParams {
    base_color_factor: vec4<f32>,
    emissive_factor: vec4<f32>,
    alpha_cutoff: f32,
    metallic: f32,
    roughness: f32,
};

@group(MATERIAL_BIND_GROUP)
@binding(0)
var<uniform> pbr_params: PbrMaterialParams;

@group(MATERIAL_BIND_GROUP)
@binding(1)
var base_color_sampler: sampler;

@group(MATERIAL_BIND_GROUP)
@binding(2)
var base_color_texture: texture_2d<f32>;

@group(MATERIAL_BIND_GROUP)
@binding(3)
var normal_texture: texture_2d<f32>;

@group(MATERIAL_BIND_GROUP)
@binding(4)
var metallic_roughness: texture_2d<f32>;

fn get_material(in: MaterialInput) -> MaterialOutput {
    var out: MaterialOutput;
    let base_color_texture_sample = textureSample(base_color_texture, base_color_sampler, in.texcoord);
    let mr = textureSample(metallic_roughness, base_color_sampler, in.texcoord);
    let color = base_color_texture_sample * pbr_params.base_color_factor * get_entity_color_or(in.entity_loc, vec4<f32>(1., 1., 1., 1.));
    out.opacity = color.a;

    out.metallic = mr.r * pbr_params.metallic;
    out.roughness = mr.g * pbr_params.roughness;

    out.alpha_cutoff = pbr_params.alpha_cutoff;
    out.base_color = color.rgb;
    out.emissive_factor = pbr_params.emissive_factor.rgb;
    out.shading = 1.;

    let normal = textureSample(normal_texture, base_color_sampler, in.texcoord).xyz * 2. - 1.;
    out.normal = in.normal_matrix * normal;
    return out;
}
