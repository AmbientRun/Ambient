struct PickerMaterialParams {
    brush_position: vec3<f32>,
    brush: i32,
    brush_radius: f32,
    brush_remapped_strength: f32,
    shape: i32,
};
@group(#MATERIAL_BIND_GROUP)
@binding(0)
var<uniform> params: PickerMaterialParams;

fn get_material(in: MaterialInput) -> MaterialOutput {

    var distance_factor = 0.0;
    if (params.shape == 0) {
        let d = length(in.world_position - params.brush_position);
        distance_factor = smoothstep(1.0, 0.0, min(1.0, d / params.brush_radius));
    } else {
        let d = abs(in.world_position - params.brush_position);
        if (d.x < params.brush_radius && d.y < params.brush_radius) {
            distance_factor = 1.0;
        }
    }

    var target_color = vec3<f32>(1.0, 1.0, 1.0);
    switch params.brush {
        case 0 { // Raise
            target_color = vec3<f32>(0.0, 1.0, 0.0);
        }
        case 1 { // Lower
            target_color = vec3<f32>(1.0, 0.0, 0.0);
        }
        case 2 { // Flatten
            target_color = vec3<f32>(1.0, 1.0, 0.0);
        }
        case 3 { // Erode
            target_color = vec3<f32>(0.0, 0.6, 0.54);
        }
        case 5 { // Thermal
            target_color = vec3<f32>(1.0, 0.4, 0.0);
        }
        default {
            // Erode2
        }
    }

    // The alpha consists of three components:
    //  - the grid falloff, which controls the appearance of a grid (lines centered around -0.1 to 0.1)
    //    this falloff is always 1.0 at the centre
    //  - the brush alpha, which shows the strength of the brush
    //  - the distance factor, which is how far away this point is from the centre of the brush
    let distance = length(in.world_position - params.brush_position);
    let grid_falloff = select(cos(clamp(fract(distance + 0.5) - 0.5, -0.1, 0.1) * 5.0 * PI), 1.0, distance < 1.0);
    let brush_alpha = mix(0.3, 0.9, params.brush_remapped_strength);
    let alpha = grid_falloff * brush_alpha * distance_factor;

    // HACK(mithun): our transparent renderer doesn't handle the blend between transparent elements correctly yet, so
    // we're opting not to render it at all under water. fix once we have some time.
    let alpha = select(alpha, 0.0, in.world_position.z < 0.0);

    var out: MaterialOutput;
    out.roughness = 1.;
    out.metallic = 1.;
    out.opacity = alpha;
    out.alpha_cutoff = 0.;
    out.base_color = target_color;
    out.emissive_factor = target_color;
    out.shading = 0.;
    out.normal = in.normal;
    return out;
}
