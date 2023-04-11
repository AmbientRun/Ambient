struct LoadingMaterialParams {
  speed: f32,
  scale: f32,
};

@group(MATERIAL_BIND_GROUP)
@binding(0)
var<uniform> loading_params: LoadingMaterialParams;

fn get_material(in: MaterialInput) -> MaterialOutput {

    let progress = sin(global_params.time * loading_params.speed - length(in.local_position) * loading_params.scale) * 0.5 + 0.5;

    var out: MaterialOutput;
    out.roughness = 0.3;
    out.metallic = 1.0;

    let color = get_entity_color_or(in.entity_loc, vec4<f32>(1., 1., 1., 1.));

    out.opacity = progress;
    out.alpha_cutoff = 0.;
    out.base_color = color.rgb;
    out.emissive_factor = vec3<f32>(1., 1., 1.);
    out.shading = 1.;
    out.normal = in.normal;
    return out;
}
