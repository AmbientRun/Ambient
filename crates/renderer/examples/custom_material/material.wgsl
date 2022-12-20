
fn get_material(in: MaterialInput) -> MaterialOutput {
    var out: MaterialOutput;
    out.opacity = in.world_position.x;
    out.alpha_cutoff = -0.1;
    out.base_color = in.normal;
    out.emissive_factor = vec3<f32>(0., 0., 0.);
    out.shading = 1.;
    out.normal = in.normal;
    out.roughness = 0.4;
    out.metallic = 0.5;
    return out;
}
