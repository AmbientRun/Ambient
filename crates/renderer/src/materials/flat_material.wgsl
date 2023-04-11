

struct FlatMaterialParams {
    color: vec4<f32>,
};
@group(MATERIAL_BIND_GROUP)
@binding(0)
var<uniform> flat_params: FlatMaterialParams;

fn get_material(in: MaterialInput) -> MaterialOutput {
    var out: MaterialOutput;
    out.roughness = 0.4;
    out.metallic = 0.5;
    let color = flat_params.color * get_entity_color_or(in.entity_loc, vec4<f32>(1., 1., 1., 1.));
    out.opacity = color.a;
    out.alpha_cutoff = 0.;
    out.base_color = color.rgb;
    out.emissive_factor = vec3<f32>(0., 0., 0.);
    out.shading = 1.;
    out.normal = in.normal;
    return out;
}
