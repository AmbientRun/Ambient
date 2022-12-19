

@group(#MATERIAL_BIND_GROUP)
@binding(0)
var font_atlas: texture_2d<f32>;

@group(#MATERIAL_BIND_GROUP)
@binding(1)
var font_sampler: sampler;

fn get_material(in: MaterialInput) -> MaterialOutput {
    var out: MaterialOutput;
    out.roughness = 0.4;
    out.metallic = 0.5;
    let color = get_entity_color_or(in.entity_loc, vec4<f32>(1., 1., 1., 1.));
    out.opacity = textureSample(font_atlas, font_sampler, in.texcoord).x * color.a;
    out.alpha_cutoff = 0.01;
    out.base_color = from_srgb_to_linear(color.rgb);
    out.emissive_factor = vec3<f32>(0., 0., 0.);
    out.shading = 1.;
    out.normal = in.normal;
    return out;
}
