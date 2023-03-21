
struct RectMaterialParams {
    background: vec4<f32>,
    border_color: vec4<f32>,
    border_radius: vec4<f32>,
    border_thickness: f32,
}
@group(MATERIAL_BIND_GROUP)
@binding(0)
var<uniform> rect_params: RectMaterialParams;

fn get_corner_from_tc(tc: vec2<f32>) -> u32 {
    var corner = 0u;
    if tc.x >= 0.5 {
        corner += 1u;
    }
    if tc.y >= 0.5 {
        corner += 2u;
    }
    return corner;
}

fn get_material(in: MaterialInput) -> MaterialOutput {
    var out: MaterialOutput;
    out.roughness = 0.4;
    out.metallic = 0.5;
    let size = get_entity_ui_size(in.entity_loc).xy;
    let p = (0.5 - abs(in.texcoord - 0.5)) * size; // Normalized to top left
    let corner = get_corner_from_tc(in.texcoord);
    let border_radius = rect_params.border_radius[corner];
    let d = distance(vec2(border_radius), p);

    let entity_color = get_entity_color_or(in.entity_loc, vec4<f32>(1., 1., 1., 1.));
    let border_color = rect_params.border_color * entity_color;
    var color = rect_params.background * entity_color;
    if max(p.x, p.y) <= border_radius {
        if d > border_radius {
            color.a = 0.;
        } else if d > border_radius - rect_params.border_thickness {
            color = border_color;
        }
    } else if min(p.x, p.y) < rect_params.border_thickness {
        color = border_color;
    }
    out.opacity = color.a;
    out.alpha_cutoff = 0.;
    out.base_color = from_srgb_to_linear(color.rgb);
    out.emissive_factor = vec3<f32>(0., 0., 0.);
    out.shading = 1.;
    out.normal = in.normal;
    return out;
}
