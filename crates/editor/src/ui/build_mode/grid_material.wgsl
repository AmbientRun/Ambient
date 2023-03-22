struct GridMaterialParams {
  major: vec2<f32>,
  minor: vec2<f32>,
  line_width: f32,
  size: f32,
};

@group(MATERIAL_BIND_GROUP)
@binding(0)
var<uniform> params: GridMaterialParams;

fn get_material(in: MaterialInput) -> MaterialOutput {


    let period = 2.0 * PI * params.major;

    let pos = in.local_position.xy * params.size;

    let major = cos(pos * period);
    let amp_offset = cos(params.line_width * 0.5 * period);
    let major = dot(max(sign(major - amp_offset), vec2<f32>(0.0)), vec2<f32>(1.0));

    let period = 2.0 * PI * params.minor;
    let minor = cos(pos * period);
    let amp_offset = cos(params.line_width * 0.1 * period);
    let minor = dot(max(sign(minor - amp_offset), vec2<f32>(0.0)), vec2<f32>(1.0));

    var out: MaterialOutput;
    out.roughness = 0.3;
    out.metallic = 1.0;

    let color = get_entity_color_or(in.entity_loc, vec4<f32>(1., 1., 1., 1.));

    out.opacity = min((major - minor * 0.5) + minor + 0.05, 1.0);
    out.alpha_cutoff = 0.;
    out.base_color = color.rgb;
    out.emissive_factor = vec3<f32>(1., 1., 1.);
    out.shading = 1.;
    out.normal = in.normal;
    return out;
}
